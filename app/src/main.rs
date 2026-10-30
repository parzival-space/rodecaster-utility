mod ui;

use thiserror::Error;
use vizia::prelude::{Application, WindowModifiers};
use crate::ui::app::{MainView};

#[derive(Error, Debug)]
enum ApplicationError {
    #[error("Vizia application error: {0}")]
    ViziaApplicationError(#[from] vizia::prelude::ApplicationError)
}

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    env_logger::init();
    
    Application::new(move |cx| {
        MainView::new(cx);
    })
        .title("RODECaster Utility")
        .run()?;

    Ok(())
}