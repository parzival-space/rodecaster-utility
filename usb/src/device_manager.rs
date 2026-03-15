use anyhow::{bail, Result};
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use hidapi::{DeviceInfo, HidApi};
use log::{error, warn};
use std::cmp::PartialEq;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub(crate) const VID_RODE: u16 = 0x19f7;
pub(crate) const PID_RODECASTER_PRO_II: &[u16] = &[0x0037, 0x0072, 0x0078, 0x0030, 0x0094, 0x0092];

#[derive(PartialEq, Debug, Clone)]
pub enum DeviceType {
    RodeCasterProII
}

#[derive(Debug, Clone)]
pub struct DeviceIdentifier {
    device_type: DeviceType,
    device_info: DeviceInfo
}

impl PartialEq for DeviceIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.device_info.vendor_id().eq(&other.device_info.vendor_id()) &&
        self.device_info.product_id().eq(&other.device_info.product_id()) &&
        self.device_info.serial_number().unwrap_or_default()
            .eq(other.device_info.serial_number().unwrap_or_default()) &&
        self.device_type == other.device_type
    }
}

#[derive(PartialEq)]
pub enum HotPlugThreadManagement {
    Quit
}

#[derive(PartialEq, Debug)]
pub enum HotPlugDeviceEvent {
    DeviceAttached(DeviceIdentifier),
    DeviceRemoved(DeviceIdentifier)
}

/// Manages connected devices and detects hotplug events for all supported RODECaster devices.
pub struct DeviceManager {
    hid_api: HidApi,
    devices: Arc<Mutex<Vec<(DeviceIdentifier)>>>,
    sender: Sender<HotPlugDeviceEvent>,
    receiver: Receiver<HotPlugThreadManagement>
}

impl DeviceManager {
    pub fn new(sender: Sender<HotPlugDeviceEvent>, receiver: Receiver<HotPlugThreadManagement>) -> Result<DeviceManager> {
        let devices = Arc::new(Mutex::new(Vec::new()));
        let devices_close = Arc::clone(&devices);
        let sender_clone = sender.clone();
        let receiver_clone = receiver.clone();

        let Ok(hid_api) = HidApi::new() else {
            bail!("Failed to initialize HID API");
        };

        // spawn new thread and run scan_devices
        thread::spawn(move || {
            let Ok(hid_api_worker) = HidApi::new() else {
                warn!("Failed to initialize HID API in worker thread. Hotplug events will not be detected.");
                return;
            };

            let mut manager = Self {
                hid_api: hid_api_worker,
                devices: devices_close,
                sender: sender_clone,
                receiver: receiver_clone
            };
            manager.run_hotplug_scan_loop();
        });

        Ok(Self { hid_api, devices, sender, receiver })
    }

    /// Returns a list of currently connected devices.
    pub fn get_devices(&self) -> Vec<DeviceIdentifier> {
        let Ok(devices) = self.devices.lock() else {
            error!("Failed to acquire lock on devices. Returning empty device list.");
            return vec![]
        };
        devices.clone()
    }

    fn add_device(&mut self, device_identifier: DeviceIdentifier) {
        let Ok(mut devices) = self.devices.lock() else {
            error!("Failed to acquire lock on devices. Cannot add new device.");
            return
        };

        if devices.iter().any(|known_device_identifier| known_device_identifier.eq(&device_identifier)) {
            warn!("Device already exists in the list of known devices. Skipping add.");
            return;
        }

        devices.push((device_identifier.clone()));
        let _ = self.sender.send(HotPlugDeviceEvent::DeviceAttached(device_identifier));
    }

    fn remove_device(&mut self, device_identifier: DeviceIdentifier) {
        let Ok(mut devices) = self.devices.lock() else {
            error!("Failed to acquire lock on devices. Cannot remove device.");
            return
        };

        devices.retain(|known_device_identifier| known_device_identifier.ne(&device_identifier));
        let _ = self.sender.send(HotPlugDeviceEvent::DeviceRemoved(device_identifier));
    }

    // scan function to detect hotplug events since HidApi doesn't support hotplugging natively yet
    fn run_hotplug_scan_loop(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(message) => {
                    if message == HotPlugThreadManagement::Quit {
                        break
                    }
                },
                Err(error) => match error {
                    TryRecvError::Empty => (),
                    TryRecvError::Disconnected => {
                        error!("Receiver has been disconnected. Stopping hotplug thread.");
                        break
                    }
                }
            };

            let mut detected_devices = vec![];
            if let Ok(_) = self.hid_api.refresh_devices() {
                for device in self.hid_api.device_list() {
                    if VID_RODE.eq(&device.vendor_id()) {

                        // RodeCaster Pro II
                        if PID_RODECASTER_PRO_II.contains(&device.product_id()) {
                            detected_devices.push(DeviceIdentifier {
                                device_info: device.clone(),
                                device_type: DeviceType::RodeCasterProII
                            });
                        }
                    }
                }
            }

            let (devices_to_add, devices_to_remove): (Vec<DeviceIdentifier>, Vec<DeviceIdentifier>) = {
                let Ok(known_devices) = self.devices.lock() else { // self is immutable for some reason
                    error!("Failed to acquire lock on devices. Skipping hotplug scan.");
                    continue
                };

                (
                    detected_devices.iter()
                        .filter(|device| !known_devices.contains(device))
                        .cloned()
                        .collect(),
                    known_devices.iter()
                        .filter(|device| !detected_devices.contains(device))
                        .cloned()
                        .collect(),
                )
            };

            for device in devices_to_add {
                self.add_device(device);
            }

            for device in devices_to_remove {
                self.remove_device(device);
            }

            sleep(Duration::from_millis(500));
        }
    }
}