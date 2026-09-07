use crossbeam::channel::{Receiver, Sender};
use vizia::{prelude, ApplicationError, include_style};
use vizia::prelude::WindowModifiers;
use crate::worker::worker::{WorkerCommand, WorkerEvent};

pub struct Application {
    worker_cmd: Sender<WorkerCommand>,
    worker_event: Receiver<WorkerEvent>,
}

impl Application {
    pub fn new(worker_cmd: Sender<WorkerCommand>, worker_event: Receiver<WorkerEvent>) -> Self {
        Self {
            worker_cmd,
            worker_event,
        }
    }

    pub fn run(&mut self) -> Result<(), ApplicationError> {
        prelude::Application::new(|cx| {
            cx.add_stylesheet(include_style!("src/ui/resources/style.css"))
                .expect("Failed to load stylesheet");

        })
            .title("RODECaster Utility")
            .inner_size((1200, 720))
            .min_inner_size(Some((1200, 720)))
            .run()
    }
}