use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use crossbeam::channel::{TryRecvError};
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use rodecaster_usb::manager::{DeviceManager, DeviceStateChanged};

///
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// The level of verbosity used for logging
    #[clap(short, long, default_value_t = LevelFilter::Info)]
    log_level: LevelFilter
}

fn main() {
    let args: Arguments = Arguments::parse();

    // init logging
    CombinedLogger::init(vec![
        TermLogger::new(args.log_level, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    ]).expect("Failed to initialize logger. Aborting.");
    info!("Starting RODECaster Utility");

    // todo: add api service (used by gui app)

    // todo: implement device handling (dummy implementation below)
    let device_manager = DeviceManager::new()
        .expect("Failed to initialize device manager. Aborting.");

    loop {
        match device_manager.try_recv() {
            Ok(DeviceStateChanged::Connected(device)) => {
                info!("New device attached: {:?}", device);
                // this is where new devices would be handled
                // dummy implementation below
            }
            Ok(DeviceStateChanged::Disconnected(device)) => {
                info!("Device removed: {:?}", device);
                // this is where devices would be removed
            }
            Err(TryRecvError::Empty) => {
                sleep(Duration::from_millis(100));
            }
            Err(TryRecvError::Disconnected) => {
                error!("Hotplug receiver disconnected. Aborting.");
                break;
            }
        }
    }
}
