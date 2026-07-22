use std::time::Duration;
use crate::commands::{CommandContext};

pub struct ListCommand {}

impl ListCommand {
    pub fn execute(context: CommandContext) {
        context.device_manager.wait_for_first_enumeration(Duration::from_secs(1))
            .expect("Failed to wait for first enumeration");

        let devices = context.device_manager.devices().iter()
            .map(|d| format!("- {} {:?}", d.serial_number, d.device_model))
            .collect::<Vec<_>>()
            .join("\n");

        println!("Found devices:\n{}", devices);
    }
}