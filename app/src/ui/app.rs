use std::ptr::null;
use std::thread;
use crossbeam::channel::{Receiver, Sender};
use log::warn;
use vizia::prelude::*;
use crate::devices::manager::DeviceEvent;
use crate::ui::state::AppState;
use crate::worker::worker::{WorkerCommand, WorkerEvent};

pub struct App {
    pub(crate) device_event_receiver: Receiver<DeviceEvent>,
}

impl App {
    pub fn run(&mut self) -> Result<(), ApplicationError> {
        let receiver = self.device_event_receiver.clone();
        Application::new(|cx| {
            AppState::default().build(cx);
            AppState::register_device_listener(cx.get_proxy(), receiver);

            cx.add_stylesheet(include_style!("src/ui/resources/style.css"))
                .expect("Failed to load stylesheet");

        })
            .title("RODECaster Utility")
            .inner_size((1200, 720))
            .min_inner_size(Some((1200, 720)))
            .run()
    }

    fn register_event_proxies(&self, cx: &mut Context) {
    }
}