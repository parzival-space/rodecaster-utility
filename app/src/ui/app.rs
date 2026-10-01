use vizia::prelude::{Application, ApplicationError, Context, Model};
use crate::ui::device_state::DeviceState;

pub struct RodecasterUtility {
}

impl RodecasterUtility {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn run(&mut self) -> Result<(), ApplicationError> {
        Application::new(|cx| {
            DeviceState::default().build(cx);

        })
            .run()
    }
}