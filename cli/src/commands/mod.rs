use clap::Subcommand;
use rodecaster_usb::manager::DeviceManager;
use crate::commands::dump::DumpCommand;
use crate::commands::update_channel::UpdateChannelArguments;

pub mod dump;
pub mod list;
pub mod update_channel;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Dumps the current internal state of the device
    Dump,

    /// List all connected supported RODECaster devices
    List,

    /// Updates the state of a channel
    UpdateChannel(UpdateChannelArguments),
}

pub struct CommandContext {
    pub device_manager: DeviceManager,
    pub serial: Option<String>,
}