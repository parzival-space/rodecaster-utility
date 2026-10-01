mod ui;
mod worker;
mod devices;

use std::thread;
use crossbeam::channel::bounded;
use log::{LevelFilter, SetLoggerError};
use simplelog::TermLogger;
use vizia::ApplicationError;
use crate::devices::manager::DeviceManager;
use crate::ui::app::App;
use crate::worker::worker::Worker;

#[derive(Debug)]
enum AppError {
    ApplicationError(ApplicationError),
    SetLoggerError(SetLoggerError)
}

fn main() -> Result<(), AppError> {
    TermLogger::init(LevelFilter::Debug, Default::default(), Default::default(), Default::default())
        .map_err(|err| AppError::SetLoggerError(err))?;

    // message bus for gui
    let (device_event_sender, device_event_receiver) = bounded(100);

    let device_manager = DeviceManager::new(device_event_sender);
    device_manager.run_new_thread();

    App {
        device_event_receiver
    }.run()
        .map_err(|err| AppError::ApplicationError(err))
}
