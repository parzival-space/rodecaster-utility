use env_logger::{Builder, Env};
use log::{debug, info};
use rodecaster_usb_rewrite::common::DeviceIdentifier;
use rodecaster_usb_rewrite::device::open_device;
use rodecaster_usb_rewrite::hotplug::{HidHotPlugEvent, HidHotPlugThreadManagement, watch_hid_hotplug_event};
use std::collections::HashMap;
use std::iter::Map;
use std::time::Duration;
use tokio::time::sleep;
use tokio::{select, spawn};

#[tokio::main]
async fn main() {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let (hotplug_tx, hotplug_rx) = flume::unbounded();
    let (management_tx, management_rx) = flume::unbounded();

    info!("Spawning hotplug task");
    let hid_handle = spawn(watch_hid_hotplug_event(hotplug_tx, management_rx));

    let mut open_handles = Vec::new();

    info!("Listening for hotplug events for 10 seconds...");
    loop {
        select! {
            Ok(event) = hotplug_rx.recv_async() => {
                info!("Hotplug event: {:?}", event);
                match event {
                    HidHotPlugEvent::DeviceAttached(device, device_type) => {
                        let handle = open_device(device).await;
                        open_handles.push(handle);
                    }
                    _ => {
                        debug!("Ignoring hotplug event: {:?}", event);
                    }
                }

            },
            _ = sleep(Duration::from_secs(10)) => {
                break;
            }
        }
    }

    info!("Shutting down hotplug task");
    let _ = management_tx.send(HidHotPlugThreadManagement::Quit);
    let _ = hid_handle.await;
}
