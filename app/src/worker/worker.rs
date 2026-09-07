use crossbeam::channel::{Receiver, Sender};

pub struct Worker {
    cmd_tx: Receiver<WorkerCommand>,
    event_rx: Sender<WorkerEvent>,
}

pub enum WorkerCommand {

}

pub enum WorkerEvent {
    DeviceListChanged,
}

impl Worker {
    pub fn new(cmd_tx: Receiver<WorkerCommand>, event_rx: Sender<WorkerEvent>) -> Self {
        Self {
            cmd_tx,
            event_rx,
        }
    }

    pub fn run(&mut self) {
    }
}