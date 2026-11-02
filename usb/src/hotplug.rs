use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use flume::{Receiver, Sender};
use futures::{select, FutureExt};
use futures::executor::block_on;
use futures_timer::Delay;
use hidapi::{HidApi};
use log::{debug, error, info};

pub(crate) const VID_RODE: u16 = 0x19f7;
pub(crate) const PID_RODECASTER_PRO_II: &[u16] = &[
    // firmware 1.6.4
    0x0037, 0x0072, 0x0078, 0x0030,
    // firmware 1.7.3
    0x0094, 0x0092
];

#[derive(Debug, Clone)]
pub enum HidHotPlugEvent {
    DeviceAttached,
    DeviceRemoved,
    ThreadStopped
}

#[derive(Debug, Clone)]
pub enum HidHotPlugThreadManagement {
    Quit
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

    loop {
        select! {
            control_event = receiver.recv_async() => match control_event {
                Ok(HidHotPlugThreadManagement::Quit) => break,
                Err(_) => {
                    error!("Receiver has been closed, terminating hot plug watcher");
                    break;
                }
            },

            // hidapi-rs is not yet async compatible, so we are currently forced to poll the
            // connected devices using a fixed interval.
            _ = Delay::new(Duration::from_millis(500)).fuse() => {
                match hidapi.refresh_devices() {
                    Err(err) => error!("Failed to refresh devices: {}", err),
                    Ok(_) => {
                        // todo: implement actual hot-plug detection
                        let devices = hidapi.device_list()
                            .map(|device| {
                                device.product_string().unwrap_or("Unknown").to_string()
                            })
                            .collect::<Vec<String>>();
                        info!("Refreshed devices: {:?}", devices);
                    },
                }
            }
        }
    }
}