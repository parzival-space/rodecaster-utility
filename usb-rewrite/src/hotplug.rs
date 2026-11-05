use crate::device::common::RodeCasterDeviceKind;
use std::collections::HashMap;
use std::ffi::CString;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use flume::{Receiver, Sender};
use futures::{select, FutureExt};
use futures::executor::block_on;
use futures_timer::Delay;
use hidapi::{DeviceInfo, HidApi};
use log::{debug, error};
use crate::common::DeviceIdentifier;
use crate::device::{RodeCasterDeviceType, RodeCasterProIIKind};

pub(crate) const VID_RODE: u16 = 0x19f7;

#[derive(Debug, Clone)]
pub enum HidHotPlugEvent {
    DeviceAttached(DeviceIdentifier, RodeCasterDeviceType),
    DeviceRemoved(DeviceIdentifier, RodeCasterDeviceType),
    ThreadStopped
}

#[derive(Debug, Clone)]
pub enum HidHotPlugThreadManagement {
    Quit
}

type KnownDeviceMap = HashMap<CString, (RodeCasterDeviceType, DeviceIdentifier)>;

fn identify_device(device_info: &DeviceInfo) -> Option<RodeCasterDeviceType> {
    if device_info.vendor_id() != VID_RODE {
        return None;
    }

    match device_info.product_id() {
        pid if RodeCasterProIIKind::PID.contains(&pid) => Some(RodeCasterProIIKind::TYPE),
        _ => None
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

    // store known devices using their paths
    let mut known_devices: KnownDeviceMap = KnownDeviceMap::new();

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
                        let connected_devices: KnownDeviceMap = hidapi.device_list()
                            .filter_map(|info| match identify_device(info) {
                                Some(device_type) => Some((info.path().to_owned(), (device_type, info.clone().into()))),
                                None => None
                            })
                            .collect();

                        // raise events for devices that are no longer connected
                        let removed_devices: Vec<_> = known_devices.keys()
                            .filter(|location| !connected_devices.contains_key(*location))
                            .cloned().collect();
                        for path in removed_devices {
                            if let Some((device_type, device_info)) = known_devices.remove(&path.clone()) {
                                let _ = sender.send_async(HidHotPlugEvent::DeviceRemoved(device_info.into(), device_type)).await;
                            }
                        }

                        // raise connected event for each device that is newly connected
                        for (path, (device_type, device_info)) in connected_devices {
                            if let None = known_devices.insert(path.clone(), (device_type, device_info.clone())) {
                                let _ = sender.send_async(HidHotPlugEvent::DeviceAttached(device_info.into(), device_type)).await;
                            }
                        }
                    },
                }
            }
        }
    }
}