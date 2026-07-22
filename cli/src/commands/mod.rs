use clap::Subcommand;
use rodecaster_usb::manager::DeviceManager;
use crate::commands::dump::{DumpCommand, DumpCommandArguments};
use crate::commands::update_channel::{UpdateChannelArguments};
use crate::commands::update_screen_brightness::{UpdateScreenBrightnessCommandArguments};
use crate::commands::update_selected_bank::{UpdateSelectedBankCommandArguments};
use crate::commands::update_soundpad::{UpdateSoundpadCommandArguments};
use crate::commands::simulate_sampler_button::{SimulateSamplerButtonArguments};

pub mod dump;
pub mod list;
pub mod update_channel;
pub mod watch;
pub mod update_screen_brightness;
pub mod update_selected_bank;
pub mod update_soundpad;
pub mod simulate_sampler_button;

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

    UpdateScreenBrightness(UpdateScreenBrightnessCommandArguments),
    UpdateSelectedBank(UpdateSelectedBankCommandArguments),
    UpdateSoundpad(UpdateSoundpadCommandArguments),
    SimulateSamplerButton(SimulateSamplerButtonArguments),
}

pub struct CommandContext {
    pub device_manager: DeviceManager,
    pub serial: Option<String>,
}