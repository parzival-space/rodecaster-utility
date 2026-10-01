use std::time::Duration;
use crossbeam::channel::{Receiver, Sender};
use crossbeam_channel::select;
use rodecaster_usb::error::UsbError;
use rodecaster_usb::manager::{DeviceIdentifier, DeviceManager};

pub struct Worker {
    cmd_rx: Receiver<WorkerCommand>,
    event_tx: Sender<WorkerEvent>,
}

pub enum WorkerCommand {
    Pong,
}

pub enum WorkerEvent {
    Ping,
    DeviceListChanged(Vec<DeviceIdentifier>),
}

impl Worker {
    pub fn new(cmd_tx: Receiver<WorkerCommand>, event_rx: Sender<WorkerEvent>) -> Self {
        Self {
            cmd_rx: cmd_tx,
            event_tx: event_rx,
        }
    }

    pub fn run(&mut self) -> Result<(), UsbError> {
        // in the end it will probably be better to extract the device manager from the usb crate
        let mut device_manager = DeviceManager::new()?;

        loop {
            select! {
                recv(self.cmd_rx) -> command => {
                    match command {
                        Ok(WorkerCommand::Pong) => {
                            log::debug!("Received ping command");
                        }
                        Err(_) => {
                            // do nothing
                        }
                    }
                }

                default(Duration::from_millis(100)) => {
                    // this currently duplicates logic and should probably be extracted from the usb crate
                    // also we currently do not check if the list was actually updated
                    self.event_tx.send(WorkerEvent::DeviceListChanged(device_manager.devices()))
                        .expect("Failed to report device list change. This should be handled in the future.");
                }
            }
        }
    }
}