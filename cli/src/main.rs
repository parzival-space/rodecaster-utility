mod commands;

use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
use rodecaster_usb::manager::DeviceManager;
use crate::commands::{Command, CommandContext};
use crate::commands::dump::DumpCommand;
use crate::commands::list::ListCommand;
use crate::commands::watch::WatchCommand;
use crate::commands::update_channel::UpdateChannelCommand;
use crate::commands::update_screen_brightness::UpdateScreenBrightnessCommand;

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
struct Arguments {
    /// The level of verbosity used for logging
    #[clap(short, long, default_value_t = LevelFilter::Error)]
    verbose: LevelFilter,

    /// The serial number of the device to connect to. Uses the first found device if not specified
    #[clap(short, long)]
    serial: Option<String>,

    #[command(subcommand)]
    command: Command
}

fn main() {
    let args = Arguments::parse();

    CombinedLogger::init(vec![
        TermLogger::new(args.verbose, Default::default(), TerminalMode::Mixed, ColorChoice::Auto)
    ]).expect("Failed to initialize logger");

    // init device manager
    let manager = DeviceManager::new()
        .expect("Failed to initialize device manager");

    let context = CommandContext {
        device_manager: manager,
        serial: args.serial,
    };

    match args.command {
        Command::Dump(arguments) => DumpCommand::execute(context, arguments),
        Command::List => ListCommand::execute(context),
        Command::Watch => WatchCommand::execute(context),
        Command::UpdateChannel(arguments) => UpdateChannelCommand::execute(context, arguments),
        Command::UpdateScreenBrightness(arguments) => UpdateScreenBrightnessCommand::execute(context, arguments),
    }
}
