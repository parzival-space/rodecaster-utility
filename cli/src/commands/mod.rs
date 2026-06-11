use clap::Subcommand;
use rodecaster_usb::manager::DeviceManager;

pub mod dump;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Dumps the current internal state of the device
    Dump,
}

pub struct CommandContext {
    pub device_manager: DeviceManager,
    pub serial: Option<String>,
}

pub trait CommandHandler {
    fn execute(context: CommandContext);
}