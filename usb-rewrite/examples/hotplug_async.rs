use env_logger::{Builder, Env};
use log::info;
use rodecaster_usb_rewrite::hotplug::{HidHotPlugThreadManagement, watch_hid_hotplug_event};
use std::time::Duration;
use tokio::time::sleep;
use tokio::{select, spawn};

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
    let _ = management_tx.send_async(HidHotPlugThreadManagement::Quit).await;
    let _ = hid_handle.await;
}
