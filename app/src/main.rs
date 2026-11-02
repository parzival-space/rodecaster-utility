mod views;
mod components;

use thiserror::Error;
use vizia::prelude::{Application, WindowModifiers};
use crate::views::main_view::MainView;

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