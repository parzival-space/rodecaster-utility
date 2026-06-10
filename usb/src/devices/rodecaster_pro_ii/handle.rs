use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam::channel::{unbounded, Receiver, Sender};
use log::{debug, error};
use crate::devices::manager::DeviceIdentifier;
use crate::devices::rodecaster_pro_ii::io::run_io_loop;
use crate::devices::rodecaster_pro_ii::state::RodeCasterProIIState;
use crate::error::UsbError;
use crate::protocol::packets::property_patch_packet::PropertyPatchPacket;
use crate::protocol::packets::device_status_packet::DeviceStatusPacket;

#[derive(Debug, Clone)]
pub enum RodeCasterProIICommand {
    SendPacket(Vec<u8>),
    Stop
}

#[derive(Debug, Clone)]
pub enum RodeCasterProIIEvent {
    Connected,
    DeviceReportReceived(DeviceStatusPacket), // todo replace with domain object
    PropertyPatchReceived(PropertyPatchPacket), // todo replace with domain object
    UnknownPacket(Vec<u8>),
    Error(String),
    Disconnect,
}

#[derive(Debug)]
pub struct RodeCasterProIIHandle {
    pub commands: Sender<RodeCasterProIICommand>,
    pub events: Receiver<RodeCasterProIIEvent>,
    state: Arc<Mutex<RodeCasterProIIState>>
}

impl RodeCasterProIIHandle {
    pub fn open(identifier: DeviceIdentifier) -> Result<Self, UsbError> {
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();

        let state = Arc::new(Mutex::new(RodeCasterProIIState::default()));

        // io thread
        let event_tx_clone = event_tx.clone();
        thread::spawn(move || {
            if let Err(err) = run_io_loop(identifier.device_info, cmd_rx, event_tx_clone.clone()) {
                event_tx_clone.send(RodeCasterProIIEvent::Error(err.to_string())).unwrap_or_default();
                error!("Error occurred while running io_loop: {}", err);
            }
        });

        // state-aggregator thread
        let event_rx_clone = event_rx.clone();
        let state_clone = state.clone();
        thread::spawn(move || {
            while let Ok(event) = event_rx_clone.recv() {
                let Ok(mut guard) = state_clone.lock() else { continue };
                match event {
                    RodeCasterProIIEvent::DeviceReportReceived(report) => guard.apply_device_report(report),
                    RodeCasterProIIEvent::PropertyPatchReceived(update) => guard.apply_property_update(update),
                    _ => {},
                }
            }
        });

        Ok(Self {
            commands: cmd_tx,
            events: event_rx,
            state,
        })
    }

    pub fn state_snapshot(&self) -> RodeCasterProIIState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }
}