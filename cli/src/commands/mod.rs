use clap::Subcommand;
use rodecaster_usb::manager::DeviceManager;
use crate::commands::dump::{DumpCommand, DumpCommandArguments};
use crate::commands::update_channel::UpdateChannelArguments;

pub mod dump;
pub mod list;
pub mod update_channel;
pub mod watch;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Dumps the current internal state of the device
    Dump(DumpCommandArguments),

    /// List all connected supported RODECaster devices
    List,

    /// Watches device communication
    Watch,

    /// Updates the state of a channel
    UpdateChannel(UpdateChannelArguments),
}

pub struct CommandContext {
    pub device_manager: DeviceManager,
    pub serial: Option<String>,
}