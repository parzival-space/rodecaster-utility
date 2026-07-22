use std::thread::sleep;
use log::{error, info};
use std::time::Duration;
use crate::commands::{CommandContext};

pub struct WatchCommand {}

impl WatchCommand {
    pub fn execute(context: CommandContext) {
        context.device_manager.wait_for_first_enumeration(Duration::from_secs(1))
            .expect("Failed to wait for first enumeration");

        let device = context.device_manager.devices().iter()
            .filter(|d| context.serial.is_none() || context.serial.clone().unwrap_or_default() == d.serial_number)
            .cloned()
            .next();
        if device.is_none() {
            error!("No device found.");
            return;
        }
        let device = device.unwrap();

        device.open(Duration::from_secs(10))
            .expect("Failed to open device");

        // wait until interrupt CTRL+C
        loop {
            sleep(Duration::from_millis(100))
        }
    }
}