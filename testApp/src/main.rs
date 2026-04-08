use crossbeam::channel::bounded;
use log::{LevelFilter, debug, error, info};
use rodecaster_usb::{DeviceManager, HotPlugDeviceEvent, OpenDeviceResult};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let (device_sender, device_receiver) = bounded(100);
    let (_control_sender, control_receiver) = bounded(100);
    let device_manager = DeviceManager::new(device_sender, control_receiver)
        .expect("Failed to create device manager");

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
                match DeviceManager::open_device(device) {
                    OpenDeviceResult::RodeCasterProII(device) => {
                        debug!(
                            "Successfully opened RodeCaster Pro II device: {:?}",
                            device.get_device_info()
                        );

                        loop {
                            // test if state actually gets updated
                            sleep(Duration::from_secs(1));
                            let Ok(state) = device.get_state() else {
                                debug!("Failed to aquire state. Exiting main loop.");
                                break;
                            };
                            info!(
                                "Current device state: {:?}",
                                state.children.first().map(|child| child
                                    .children
                                    .first()
                                    .map(|childchild| &childchild.properties))
                            );
                        }
                    }
                    OpenDeviceResult::Err(error) => {
                        error!("Failed to open RodeCaster Pro II device: {:?}", error)
                    }
                }
            }
            HotPlugDeviceEvent::DeviceRemoved(_) => {}
        }
    }

    sleep(Duration::from_secs(1000));
}
