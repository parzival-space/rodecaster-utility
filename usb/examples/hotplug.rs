use std::thread::sleep;
use std::time::{Duration, Instant};
use env_logger::{Builder, Env};
use flume::TryRecvError;
use log::info;
use rodecaster_usb::hotplug::{spawn_hotplug_handler, HidHotPlugThreadManagement};

fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let (hotplug_tx, hotplug_rx) = flume::unbounded();
    let (management_tx, management_rx) = flume::unbounded();

    info!("Spawning hotplug thread");
    let hid_handle = spawn_hotplug_handler(hotplug_tx, management_rx);

    info!("Listening for hotplug events for 10 seconds...");
    let start = Instant::now();
    loop {
        match hotplug_rx.try_recv() {
            Ok(event) => {
                info!("Hotplug event: {:?}", event);
            },
            Err(e) => {
                if e == TryRecvError::Empty {
                    if start.elapsed() > Duration::from_secs(10) {
                        // Timeout reached, exit the loop
                        break;
                    }

                    sleep(Duration::from_millis(10));
                    continue;
                }
            }
        }
    }

    info!("Shutting down hotplug task");
    let _ = management_tx.send(HidHotPlugThreadManagement::Quit);
    hid_handle.join().unwrap_or_default()
}