use vizia::{Application, ApplicationError};
use vizia::model::Model;
use vizia::prelude::WindowModifiers;
use crate::ui::state::AppState;

pub fn run_app(state: AppState) -> Result<(), ApplicationError> {
    Application::new(|cx| {
        state.build(cx);
        
        // root view
        
    })
        .title("RODECaster Utility")
        .inner_size((1200, 720))
        .min_inner_size(Some((1200, 720)))
        .run()
}