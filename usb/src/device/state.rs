use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use log::{trace, warn};
use crate::device::io::IoRxEvent;
use crate::error::UsbError;
use crate::protocol::types::composite::Composite;

pub(crate) fn run_state_loop(
    state: Arc<Mutex<Composite>>,
    io_receiver: Receiver<IoRxEvent>,
    ready_sender: Sender<()>,
    shutdown_requested: Arc<AtomicCell<bool>>
) -> Result<(), UsbError> {
    loop {
        if shutdown_requested.load() {
            break;
        }

        match io_receiver.try_recv() {
            Ok(IoRxEvent::DeviceReportReceived(report)) => {
                if let Ok(mut state) = state.lock() {
                    *state = report.get_status().to_owned();

                    // send ready signal, indicating that the first status update has been received
                    ready_sender.send(()).unwrap_or_default();
                }
            },
            Ok(IoRxEvent::PropertyPatchReceived(patch)) => {
                trace!("Received property patch: {:?}", patch);
                if let Ok(mut state) = state.lock() {
                    if let Err(err) = state.apply_patch(
                        patch.get_indices().to_owned(),
                        patch.get_name().to_owned(),
                        patch.get_value().to_owned()) {
                        warn!("Failed to apply status patch: {}", err);
                    }
                }
            },
            Ok(IoRxEvent::Disconnect) => {
                warn!("IO thread disconnected, no longer receiving status updates");
                break;
            },
            Ok(IoRxEvent::UnknownPacket(packet)) => {
                // todo: not sure if this belongs here
                warn!("Received unknown packet: {:02X?}", packet);
            },
            Err(TryRecvError::Empty) | Ok(IoRxEvent::Error(_)) => {
                sleep(Duration::from_millis(2));
                continue;
            }
            Err(TryRecvError::Disconnected) => break,
        }
    }
    Ok(())
}