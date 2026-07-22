use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use log::{error, info};
use crate::commands::{Command, CommandContext};
use crate::commands::dump::DumpCommand;
use rodecaster_usb::protocol::types::value::Value;

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
pub struct UpdateChannelArguments {
    #[clap(short, long)]
    pub channel: i8,

    #[clap(short, long)]
    pub mute: bool,

    #[clap(short, long)]
    pub solo: bool,
}

pub struct UpdateChannelCommand {}
impl UpdateChannelCommand {
    pub fn execute(context: CommandContext, arguments: UpdateChannelArguments) {
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

        // calculate the channel offset
        let channel_offset = device_handle.state_snapshot().children()
            .iter().position(|c| c.get_name().contains("CHANNEL"))
            .expect("Failed to find channel offset");
        let channel_position = channel_offset + arguments.channel as usize;

        device_handle.send_patch(
            vec![channel_position],
            "channelOutputMute".to_string(),
            Value::Bool(arguments.mute)
        ).unwrap_or_default();
        device_handle.send_patch(
            vec![channel_position],
            "channelCueEnable".to_string(),
            Value::Bool(arguments.solo)
        ).unwrap_or_default();

        // wait until for changes to apply
        sleep(Duration::from_millis(500))
    }
}