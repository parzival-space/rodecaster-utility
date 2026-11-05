use crate::device::common::RodeCasterDeviceKind;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use flume::{Receiver, Sender};
use futures::{select, FutureExt};
use futures::executor::block_on;
use futures_timer::Delay;
use hidapi::{DeviceInfo, HidApi};
use log::{debug, error};
use crate::device::{RodeCasterDeviceType, RodeCasterProIIKind};

pub(crate) const VID_RODE: u16 = 0x19f7;

#[derive(Debug, Clone)]
pub enum HidHotPlugEvent {
    DeviceAttached(String, RodeCasterDeviceType),
    DeviceRemoved(String, RodeCasterDeviceType),
    ThreadStopped
}

#[derive(Debug, Clone)]
pub enum HidHotPlugThreadManagement {
    Quit
}

// used to store device information internally
#[derive(Debug, Clone)]
struct KnownDevice {
    location: String,
    device_type: RodeCasterDeviceType
}

impl TryFrom<DeviceInfo> for KnownDevice {
    // todo: introduce proper error type for TryFrom mapping
    type Error = ();

    fn try_from(device_info: DeviceInfo) -> Result<Self, Self::Error> {
        if device_info.vendor_id() != VID_RODE {
            // wrong vendor id
            return Err(());
        };

        match device_info.product_id() {
            pro_ii if RodeCasterProIIKind::PID.contains(&pro_ii) => {
                Ok(KnownDevice {
                    location: device_info.path().to_str().unwrap_or_default().to_string(),
                    device_type: RodeCasterProIIKind::TYPE
                })
            },
            _ => Err(())
        }
    }
}

/// Spawns an OS thread and watches for HID device hot-plug events.
/// Use `watch_hid_hotplug_event(sender, receiver)` if you want to use an async task.
/// Runs until `receiver` receives `HidHotPlugThreadManagement::Quit`.
pub fn spawn_hotplug_handler(
    sender: Sender<HidHotPlugEvent>,
    receiver: Receiver<HidHotPlugThreadManagement>,
) -> JoinHandle<()> {
    debug!("Spawning hotplug handler thread");
    thread::spawn(|| block_on(watch_hid_hotplug_event(sender, receiver)))
}

/// Watch for HID hotplug events and send them to the `sender` using async task.
/// Can be used in whatever async runtime you want to use.
/// Runs until `receiver` receives `HidHotPlugThreadManagement::Quit`.
pub async fn watch_hid_hotplug_event(
    sender: Sender<HidHotPlugEvent>,
    receiver: Receiver<HidHotPlugThreadManagement>,
) {
    // initialize HidApi
    let mut hidapi = match HidApi::new() {
        Ok(hidapi) => hidapi,
        Err(err) => {
            eprintln!("Failed to initialize HidApi: {}", err);
            return;
        }
    };

    // store known devices
    let mut known_devices: HashMap<String, KnownDevice> = HashMap::new();

    loop {
        select! {
            control_event = receiver.recv_async() => match control_event {
                Ok(HidHotPlugThreadManagement::Quit) => break,
                Err(_) => {
                    error!("Receiver has been closed, terminating hot plug watcher");
                    let _ = sender.send_async(HidHotPlugEvent::ThreadStopped).await;
                    break;
                }
            },

            // hidapi-rs is not yet async compatible, so we are currently forced to poll the
            // connected devices using a fixed interval.
            _ = Delay::new(Duration::from_millis(500)).fuse() => {
                match hidapi.refresh_devices() {
                    Err(err) => error!("Failed to refresh devices: {}", err),
                    Ok(_) => {
                        // get all currently connected known devices
                        let connected_devices: HashMap<String, KnownDevice> = hidapi.device_list()
                            .filter_map(|device| {
                                match KnownDevice::try_from(device.to_owned()) {
                                    Ok(device) => Some((device.location.clone(), device)),
                                    Err(_) => None
                                }
                            })
                            .collect();

                        let removed_locations: Vec<_> = known_devices.keys()
                            .filter(|location| !connected_devices.contains_key(*location))
                            .cloned().collect();
                        for location in removed_locations {
                            if let Some(device) = known_devices.remove(&location.clone()) {
                                let _ = sender.send_async(HidHotPlugEvent::DeviceRemoved(location, device.device_type)).await;
                            }
                        }

                        // raise connected event for each device that is newly connected
                        for (location, device) in connected_devices {
                            if let None = known_devices.insert(location.clone(), device.clone()) {
                                let _ = sender.send_async(HidHotPlugEvent::DeviceAttached(location.clone(), device.device_type.clone())).await;
                            }
                        }
                    },
                }
            }
        }
    }
}