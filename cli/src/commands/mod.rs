use clap::Subcommand;
use rodecaster_usb::manager::DeviceManager;

pub mod dump;
pub mod list;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Dumps the current internal state of the device
    Dump,

    /// List all connected supported RODECaster devices
    List,
}

pub struct CommandContext {
    pub device_manager: DeviceManager,
    pub serial: Option<String>,
}

pub trait CommandHandler {
    fn execute(context: CommandContext);
}