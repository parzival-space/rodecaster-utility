use log::{LevelFilter, SetLoggerError};
use simplelog::{Config, TermLogger};
use slint::PlatformError;

slint::include_modules!();

#[derive(Debug)]
enum RodeCasterUtilityError {
    PlatformError(PlatformError),
    SetLoggerError(SetLoggerError)
}

fn main() -> Result<(), RodeCasterUtilityError> {
    TermLogger::init(LevelFilter::Debug, Default::default(), Default::default(), Default::default())
        .map_err(|err| RodeCasterUtilityError::SetLoggerError(err))?;

    let main_window = MainWindow::new()
        .map_err(|err| RodeCasterUtilityError::PlatformError(err))?;
    main_window.run()
        .map_err(|err| RodeCasterUtilityError::PlatformError(err))
}
