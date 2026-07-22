use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use log::{debug, error, info};
use rodecaster_usb::protocol::types::value::Value;
use crate::commands::CommandContext;

pub struct SimulateSamplerButtonCommand {}

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
pub struct SimulateSamplerButtonArguments {
    /// Sample button, top to bottom, left to right, 0 - 7
    #[clap(short, long)]
    button: u8,

    /// Press or release the button
    #[clap(short, long)]
    pressed: bool,
}

impl SimulateSamplerButtonCommand {
    pub fn execute(context: CommandContext, arguments: SimulateSamplerButtonArguments) {
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
        let physical_offset = {
            device_handle.state_snapshot().children()
                .iter().position(|c| c.get_name().contains("PHYSICALINTERFACE"))
                .expect("Failed to find channel offset").clone()
        };
        let padbutton_offset = {
            device_handle.state_snapshot().children()
                .iter().find(|c| c.get_name().contains("PHYSICALINTERFACE"))
                .expect("Failed to open PHYSICALINTERFACE").clone()
                .children().iter().position(|c| c.get_name().contains("PADBUTTON"))
                .expect("Failed to find PADBUTTON offset")
        };
        let index = padbutton_offset + arguments.button as usize;
        debug!("Using indices: {}, {}", physical_offset, index);

        device_handle.send_patch(
            vec![physical_offset, index],
            "padButtonPressed".to_string(),
            Value::Bool(arguments.pressed)
        ).unwrap_or_default();

        // wait until for changes to apply
        sleep(Duration::from_millis(500))
    }
}