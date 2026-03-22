use std::thread::sleep;
use std::time::Duration;
use log::{debug, error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use rodecaster_usb::{DeviceManager, HotPlugDeviceEvent, OpenDeviceResult};
use crossbeam::channel::{bounded, };

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();

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
        info!("Current connected devices: {:?}", device_manager.get_devices().len());

        match hotplug_event {
            HotPlugDeviceEvent::DeviceAttached(device) => {
                match DeviceManager::open_device(device) {
                    OpenDeviceResult::RodeCasterProII(device) => {
                        debug!("Successfully opened RodeCaster Pro II device: {:?}", device.get_device_info());
                    }
                    OpenDeviceResult::Err(error) => error!("Failed to open RodeCaster Pro II device: {:?}", error),
                }
            }
            HotPlugDeviceEvent::DeviceRemoved(_) => {

            },
        }
    }


    sleep(Duration::from_secs(1000));
}