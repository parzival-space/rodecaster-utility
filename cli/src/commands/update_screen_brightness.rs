use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use log::{error, info};
use rodecaster_usb::protocol::types::value::Value;
use crate::commands::CommandContext;

pub struct UpdateScreenBrightnessCommand {}

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
pub struct UpdateScreenBrightnessCommandArguments {
    pub brightness: u32,
}

impl UpdateScreenBrightnessCommand {
    pub fn execute(context: CommandContext, arguments: UpdateScreenBrightnessCommandArguments) {
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

        let device_handle = device.open(Duration::from_secs(10))
            .expect("Failed to open device");

        // calculate the offset
        let offset = device_handle.state_snapshot().children()
            .iter().position(|c| c.get_name().contains("PHYSICALINTERFACE"))
            .expect("Failed to find channel offset");

        device_handle.send_patch(
            vec![offset],
            "hwScreenBrightness".to_string(),
            Value::U32(arguments.brightness)
        ).unwrap_or_default();

        // wait until for changes to apply
        sleep(Duration::from_millis(500))
    }
}