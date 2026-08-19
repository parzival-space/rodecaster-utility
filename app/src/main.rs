mod ui;

use crate::ui::app::run_app;
use crate::ui::state::AppState;
use log::{LevelFilter, SetLoggerError};
use simplelog::TermLogger;
use vizia::ApplicationError;

#[derive(Debug)]
enum AppError {
    ApplicationError(ApplicationError),
    SetLoggerError(SetLoggerError)
}

fn main() -> Result<(), AppError> {
    TermLogger::init(LevelFilter::Debug, Default::default(), Default::default(), Default::default())
        .map_err(|err| AppError::SetLoggerError(err))?;

    let state = AppState::default(); // todo: use constructor later

    run_app(state)
        .map_err(|err| AppError::ApplicationError(err))
}
