use std::time::Duration;
use env_logger::{Builder, Env};
use log::info;
use tokio::{select, spawn};
use tokio::time::sleep;
use rodecaster_usb::hotplug::{watch_hid_hotplug_event, HidHotPlugThreadManagement};

#[tokio::main]
async fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let (hotplug_tx, hotplug_rx) = flume::unbounded();
    let (management_tx, management_rx) = flume::unbounded();

    info!("Spawning hotplug task");
    let hid_handle = spawn(watch_hid_hotplug_event(hotplug_tx, management_rx));

    info!("Listening for hotplug events for 10 seconds...");
    loop {
        select! {
            Ok(event) = hotplug_rx.recv_async() => {
                info!("Hotplug event: {:?}", event);
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
