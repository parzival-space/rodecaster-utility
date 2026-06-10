use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use crossbeam::channel::{bounded, TryRecvError};
use log::{debug, error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use rodecaster_usb::devices::manager::{DeviceManager, HotPlugDeviceEvent};
use rodecaster_usb::devices::open::open_device;

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
    let (hotplug_sender, hotplug_receiver) = bounded(100);
    let (_control_sender, control_receiver ) = bounded(100);
    let device_manager = DeviceManager::new(hotplug_sender, control_receiver);

    loop {
        match hotplug_receiver.try_recv() {
            Ok(HotPlugDeviceEvent::DeviceAttached(device)) => {
                info!("New device attached: {:?}", device);
                // this is where new devices would be handled
                // dummy implementation below
                match open_device(device) {
                    Ok(device) => {
                        debug!("Opened device: {:?}", device);
                        loop {
                            // again dummy implementation, just to keep the connetion alive for now
                            sleep(Duration::from_secs(1));
                        }
                    }
                    Err(e) => {
                        error!("Failed to open device: {:?}", e);
                    }
                }
            }
            Ok(HotPlugDeviceEvent::DeviceRemoved(device)) => {
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
