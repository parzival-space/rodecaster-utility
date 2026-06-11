use std::time::Duration;
use log::{error, info};
use rodecaster_usb::manager::DeviceManager;
use crate::commands::{CommandContext, CommandHandler};

pub struct DumpCommand {}

impl CommandHandler for DumpCommand {
    fn execute(context: CommandContext) {
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

        info!("Found {:?} with serial number {:?}", device.device_model, device.serial_number);
    }
}