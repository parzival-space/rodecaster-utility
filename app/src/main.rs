use crate::ui::app::RodecasterUtility;
use thiserror::Error;

mod ui;

#[derive(Error, Debug)]
enum MainError {
    #[error("Vizia application error: {0}")]
    ApplicationError(#[from] vizia::prelude::ApplicationError)
}

fn main() -> Result<(), MainError> {
    env_logger::init();

    let mut ui = RodecasterUtility::new();
    ui.run()
        .map_err(|err| MainError::from(err))
}
