mod ui;
mod worker;

use std::thread;
use crossbeam::channel::bounded;
use log::{LevelFilter, SetLoggerError};
use simplelog::TermLogger;
use vizia::ApplicationError;
use crate::ui::app::Application;
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
    let (worker_command_tx, worker_command_rx) = bounded(100);
    let (worker_event_tx, worker_event_rx) = bounded(100);
    thread::spawn(move || {
        let mut worker = Worker::new(worker_command_rx, worker_event_tx);
        worker.run();
    });

    let mut application = Application::new(worker_command_tx, worker_event_rx);
    application.run()
        .map_err(|err| AppError::ApplicationError(err))
}
