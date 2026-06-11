use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use hidapi::{DeviceInfo, HidApi};
use log::{error, warn};
use crate::device::model::DeviceModel;
use crate::error::UsbError;

/// Contains information about a connected device.
#[derive(Debug, Clone)]
pub struct DeviceIdentifier {
    pub device_model: DeviceModel,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: String,
    pub version: u16,
    pub(crate) device_info: DeviceInfo
}

impl PartialEq for DeviceIdentifier {
    fn eq(&self, other: &DeviceIdentifier) -> bool {
        self.vendor_id == other.vendor_id &&
            self.product_id == other.product_id &&
            self.serial_number == other.serial_number &&
            self.version == other.version &&
            self.device_model == other.device_model
        // ignore device_info field
    }
}

impl From<DeviceInfo> for DeviceIdentifier {
    fn from(value: DeviceInfo) -> Self {
        let serial_number = value.serial_number().unwrap_or_default();
        Self {
            device_model: DeviceModel::ProII, // todo: implement device_model detection
            vendor_id: value.vendor_id(),
            product_id: value.product_id(),
            serial_number: value.serial_number().unwrap_or_default().to_string(),
            version: value.release_number(), // todo: implement version detection
            device_info: value.clone(),
        }
    }
}

#[derive(Debug)]
pub enum DeviceStateChanged {
    Connected(DeviceIdentifier),
    Disconnected(DeviceIdentifier),
}

pub struct DeviceManager {
    devices: Arc<Mutex<Vec<DeviceIdentifier>>>,

    // thread control
    shutdown_requested: Arc<AtomicCell<bool>>,
    receiver: Receiver<DeviceStateChanged>,
    device_scan_thread: Option<thread::JoinHandle<()>>,
}

impl DeviceManager {
    pub fn new() -> Result<Self, UsbError> {
        let devices = Arc::new(Mutex::new(Vec::new()));
        let shutdown_requested = Arc::new(AtomicCell::new(false));
        let (sender, receiver) = unbounded();

        // spawn device scan thread
        let devices_clone = devices.clone();
        let shutdown_requested_clone = shutdown_requested.clone();
        let device_scan_thread = thread::Builder::new()
            .name("device-scan-thread".into())
            .spawn(move || {
                if let Err(err) = Self::scan_devices(devices_clone.clone(), sender, shutdown_requested_clone) {
                    error!("Device scan thread failed: {}", err);
                }
            })?;

        Ok(Self {
            devices,
            shutdown_requested,
            receiver,
            device_scan_thread: Some(device_scan_thread),
        })
    }

    pub fn close(&mut self) -> Result<(), UsbError> {
        if self.shutdown_requested.load() {
            // already shutting down
            return Ok(());
        }

        self.shutdown_requested.store(true);
        if let Some(device_scan_thread) = self.device_scan_thread.take() {
            device_scan_thread.join().unwrap_or_default();
        }
        Ok(())
    }
    
    pub fn try_recv(&self) -> Result<DeviceStateChanged, TryRecvError> {
        self.receiver.try_recv()
    }

    fn scan_devices(devices: Arc<Mutex<Vec<DeviceIdentifier>>>, sender: Sender<DeviceStateChanged>, shutdown_requested: Arc<AtomicCell<bool>>) -> Result<(), UsbError> {
        let mut hid_api = HidApi::new()?;

        loop {
            if shutdown_requested.load() {
                break;
            }

            if hid_api.refresh_devices().is_err() {
                warn!("Failed to refresh devices");
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            let detected: Vec<DeviceIdentifier> = hid_api.device_list() // only rode devices
                .filter(|d| DeviceModel::from_device_info(d.to_owned().to_owned()).is_some()) // todo: move this into model info
                .map(|d| DeviceIdentifier::from(d.to_owned()))
                .collect();
            
            let known_devices = match devices.lock() { 
                Ok(devices) => devices.clone(),
                Err(err) => {
                    error!("Failed to lock devices: {}", err);
                    continue;
                }
            };
            
            // sync devices list
            for device in detected.iter()
                .filter(|d| !known_devices.contains(d)).cloned() {
                if let Ok(mut guard) = devices.lock() {
                    guard.push(device.clone());
                    sender.send(DeviceStateChanged::Connected(device)).unwrap_or_default();
                }
            }
            for device in known_devices.iter()
                .filter(|d| !detected.contains(d)).cloned() {
                if let Ok(mut guard) = devices.lock() {
                    guard.retain(|d| d != &device);
                    sender.send(DeviceStateChanged::Disconnected(device)).unwrap_or_default();
                }
            }
            
            sleep(Duration::from_millis(100));
        }

        Ok(())
    }
}