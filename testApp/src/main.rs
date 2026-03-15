use std::fs;
use std::thread::sleep;
use std::time::Duration;
use log::{debug, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use rodecaster_usb::{DeviceManager, DeviceIdentifier};
use crossbeam::channel::{bounded, };

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();

    let (device_sender, device_receiver) = bounded(100);
    let (control_sender, control_receiver) = bounded(100);
    let device_manager = DeviceManager::new(device_sender, control_receiver)
        .expect("Failed to create device manager");

    loop {
        let Ok(new_device) = device_receiver.recv() else {
            debug!("Device manager thread has been terminated. Exiting main loop.");
            break;
        };

        info!("New device received: {:?}", new_device);
        info!("Current connected devices: {:?}", device_manager.get_devices().len());
    }


    sleep(Duration::from_secs(1000));
}