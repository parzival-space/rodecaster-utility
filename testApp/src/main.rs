use std::os::linux::raw::stat;
use crossbeam::channel::bounded;
use log::{LevelFilter, debug, error, info};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::thread::sleep;
use std::time::Duration;
use rodecaster_usb::devices::manager::{DeviceManager, HotPlugDeviceEvent};
use rodecaster_usb::devices::open::{open_device, DeviceHandle};

fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let (device_sender, device_receiver) = bounded(100);
    let (_control_sender, control_receiver) = bounded(100);
    let device_manager = DeviceManager::new(device_sender, control_receiver).unwrap();

    loop {
        let Ok(hotplug_event) = device_receiver.recv() else {
            debug!("Device manager thread has been terminated. Exiting main loop.");
            break;
        };

        info!("New device received: {:?}", hotplug_event);
        info!(
            "Current connected devices: {:?}",
            device_manager.get_devices().len()
        );

        match hotplug_event {
            HotPlugDeviceEvent::DeviceAttached(device) => {
                match open_device(device) {
                    Ok(DeviceHandle::RodeCasterProII(device)) => {
                        debug!(
                            "Successfully opened RodeCaster Pro II device: {:?}",
                            device.state_snapshot()
                        );

                        loop {
                            // test if state actually gets updated
                            sleep(Duration::from_secs(1));
                            let state = device.state_snapshot();
                            if let Some(root) = state.root {
                                info!(
                                "Current device state: {:?}",
                                root.children.first().map(|child| child
                                    .children
                                    .first()
                                    .map(|childchild| &childchild.properties))
                                );
                            } else {
                                error!("No RodeCasterProII state found");
                            }
                        }
                    }
                    Ok(_) => {
                        // do nothing
                    }
                    Err(error) => {
                        error!("Failed to open RodeCaster Pro II device: {:?}", error)
                    }
                }
            }
            HotPlugDeviceEvent::DeviceRemoved(_) => {}
        }
    }

    sleep(Duration::from_secs(1000));
}
