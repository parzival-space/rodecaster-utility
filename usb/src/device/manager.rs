use std::cmp::PartialEq;
use std::sync::{Arc, Mutex};
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use hidapi::{DeviceInfo, HidApi};
use log::{error, warn};
use crate::devices::KNOWN_DEVICES;
use crate::devices::DeviceType;
use crate::error::UsbError;

#[derive(Debug, Clone)]
pub struct DeviceIdentifier {
    pub device_type: DeviceType,
    pub(crate) device_info: DeviceInfo,
}

impl PartialEq for DeviceIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.device_type == other.device_type
            && self.device_info.vendor_id() == other.device_info.vendor_id()
            && self.device_info.product_id() == other.device_info.product_id()
            && self.device_info.serial_number().unwrap_or_default()
                == other.device_info.serial_number().unwrap_or_default()
    }
}

#[derive(Debug, PartialEq)]
pub enum HotPlugDeviceEvent {
    DeviceAttached(DeviceIdentifier),
    DeviceRemoved(DeviceIdentifier),
}

#[derive(Debug, PartialEq)]
pub enum HotPlugThreadManagement {
    Quit,
}

pub struct DeviceManager {
    hid_api: HidApi,
    devices: Arc<Mutex<Vec<DeviceIdentifier>>>,
    sender: Sender<HotPlugDeviceEvent>,
    receiver: Receiver<HotPlugThreadManagement>,
}

impl DeviceManager {
    pub fn new(
        sender: Sender<HotPlugDeviceEvent>,
        receiver: Receiver<HotPlugThreadManagement>
    ) -> Result<Self, UsbError> {
        let hid_api = HidApi::new()?;
        let devices = Arc::new(Mutex::new(Vec::new()));

        let mut worker = Self {
            hid_api: HidApi::new()?,
            devices: Arc::clone(&devices),
            sender: sender.clone(),
            receiver: receiver.clone(),
        };
        thread::spawn(move || worker.run_hotplug_scan_loop());

        Ok(Self {
            hid_api,
            devices,
            sender,
            receiver,
        })
    }

    pub fn get_devices(&self) -> Vec<DeviceIdentifier> {
        self.devices.lock().map(|v| v.clone()).unwrap_or_default()
    }

    fn detect_device(device: &DeviceInfo) -> Option<DeviceType> {
        KNOWN_DEVICES.iter().find_map(|entry| {
            (entry.vendor_id == device.vendor_id() && entry.product_ids.contains(&device.product_id()))
                .then(|| entry.device_type.clone())
        })
    }

    fn run_hotplug_scan_loop(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(HotPlugThreadManagement::Quit) => break,
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => (),
            }

            if self.hid_api.refresh_devices().is_err() {
                warn!("Failed to refresh devices");
                sleep(Duration::from_millis(500));
                continue;
            }

            let detected: Vec<DeviceIdentifier> = self
                .hid_api
                .device_list()
                .filter_map(|d| Self::detect_device(d).map(|t| DeviceIdentifier {
                    device_type: t,
                    device_info: d.clone(),
                }))
                .collect();

            // compare detected devices to known devices
            let known_devices = match self.devices.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => {
                    error!("Failed to acquire lock on devices. Skipping hotplug scan.");
                    continue;
                }
            };

            // sync device states
            for device in detected.iter()
                .filter(|d| !known_devices.contains(d)).cloned() {
                // devices to add
                if let Ok(mut guard) = self.devices.lock() {
                    guard.push(device.clone());
                }
                self.sender.send(HotPlugDeviceEvent::DeviceAttached(device)).unwrap_or_default();
            }

            for device in known_devices.iter()
                .filter(|d| !detected.contains(d)).cloned() {
                // devices to remove
                if let Ok(mut guard) = self.devices.lock() {
                    guard.retain(|d| d != &device);
                }
                self.sender.send(HotPlugDeviceEvent::DeviceRemoved(device)).unwrap_or_default();
            }

            sleep(Duration::from_millis(500));
        }
    }
}