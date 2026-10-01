// this slowly replaces the old hid device manager

use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::Sender;
use hidapi::{DeviceInfo, HidApi};
use log::{debug, error, info};

pub enum DeviceEvent {
    DeviceAttached(DeviceInfo),
    DeviceRemoved(DeviceInfo),
}

#[derive(Debug, Clone)]
pub struct DeviceManager {
    devices: Arc<Mutex<Vec<DeviceInfo>>>,

    event_sender: Sender<DeviceEvent>
}

impl DeviceManager {
    pub fn new(event_sender: Sender<DeviceEvent>) -> Self {
        Self {
            devices: Default::default(),
            event_sender
        }
    }

    pub fn run_new_thread(&self) {
        let devices = self.devices.clone();
        let event_sender = self.event_sender.clone();
        thread::spawn(move || {
            debug!("Device manager thread started");
            Self::hid_thread(devices, event_sender);
        });
    }

    fn hid_thread(devices: Arc<Mutex<Vec<DeviceInfo>>>, event_sender: Sender<DeviceEvent>) {
        let mut hid_api = HidApi::new().unwrap(); // fixme: add error handling

        loop {
            // refresh device list
            if hid_api.refresh_devices().is_ok() {
                let detected_devices: Vec<_> = hid_api.device_list()
                    .map(|dd| dd.to_owned()).collect();
                let known_devices = match devices.lock() {
                    Ok(devices) => devices.clone(),
                    Err(_) => detected_devices.clone()
                };

                // handle attached devices
                for attached_device in detected_devices.iter()
                    .filter(|dd| !known_devices.iter().any(|kd| kd.path() == dd.path())) {
                    if let Ok(mut guard) = devices.lock() {
                        guard.push(attached_device.to_owned());
                    }

                    if let Err(err) = event_sender.send(DeviceEvent::DeviceAttached(attached_device.to_owned())) {
                        error!("Failed to send DeviceAttached event: {}", err);
                    }
                }

                // handle removed devices
                for removed_device in known_devices.iter()
                    .filter(|kd| !detected_devices.iter().any(|dd| dd.path() == kd.path())) {
                    if let Ok(mut guard) = devices.lock() {
                        guard.retain(|kd| kd.path() != removed_device.path());
                    }

                    if let Err(err) = event_sender.send(DeviceEvent::DeviceRemoved(removed_device.to_owned())) {
                        error!("Failed to send DeviceRemoved event: {}", err);
                    }
                }
            }

            sleep(Duration::from_millis(100));
        }
    }
}

