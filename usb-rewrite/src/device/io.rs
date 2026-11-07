use crate::RodeCasterError;
use crate::device::common::RodeCasterDeviceKind;
use crate::protocol::hid_transport::{RodeCasterHidTransport, RodeCasterHidTransportError};
use flume::{Receiver, RecvTimeoutError, Sender};
use hidapi::{DeviceInfo, HidApi, HidDevice, HidError, HidResult};
use log::{debug, error, trace, warn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;

pub(crate) enum RodeCasterDeviceIOEvent {
    PacketReceived(Vec<u8>),
    ThreadQuit
}

pub(crate) trait RodeCasterDeviceIOHandler {
    async fn spawn_io_handler(
        device_info: DeviceInfo,
        rx_sender: Sender<RodeCasterDeviceIOEvent>,
        tx_receiver: Receiver<RodeCasterDeviceIOEvent>,
        cancel: Arc<AtomicBool>
    ) {
        let rx_info = device_info.clone();
        let rx_cancel = Arc::clone(&cancel);
        let tx_info = device_info.clone();
        let tx_cancel = Arc::clone(&cancel);

        #[cfg(feature = "tokio")]
        {
            use tokio::runtime::Handle;
            if let Ok(handle) = Handle::try_current() {
                debug!("Starting IO background tasks");
                handle.spawn_blocking(move || Self::run_rx_handler(rx_info, rx_sender, rx_cancel));
                handle.spawn_blocking(move || Self::run_tx_handler(tx_info, tx_receiver, tx_cancel));
                return;
            }
        }

        // try to find name in this order: product string > serial number > device path > "unknown"
        let device_name = device_info.product_string().unwrap_or(device_info.serial_number().unwrap_or(device_info.path().to_str().unwrap_or("unknown")));

        use std::thread::Builder;
        let _ = Builder::new()
            .name(format!("{}-rx", device_name))
            .spawn(move || Self::run_rx_handler(rx_info, rx_sender, rx_cancel));
        let _ = Builder::new()
            .name(format!("{}-tx", device_name))
            .spawn(move || Self::run_tx_handler(tx_info, tx_receiver, tx_cancel));
    }

    // reads the device input and issues internal events that the protocol implementation can parse
    fn run_rx_handler(
        device_info: DeviceInfo,
        sender: Sender<RodeCasterDeviceIOEvent>,
        canceled: Arc<AtomicBool>
    ) {
        let transport = match RodeCasterHidTransport::new(
            device_info,
            0x03,
            0x04
        ) {
            Ok(transport) => transport,
            Err(e) => {
                error!("Failed to create HID transport: {}", e);
                return;
            }
        };

        while !canceled.load(Ordering::Relaxed) {
            match transport.read_timeout(Duration::from_millis(100)) {
                Ok(packet) => {
                    trace!("Received packet from HID device: {:?}", packet);
                    let _ = sender.send(RodeCasterDeviceIOEvent::PacketReceived(packet));
                }
                Err(err) => match err {
                    RodeCasterHidTransportError::TimeoutError => {
                        // just a timeout error, nothing to do here
                    },
                    err => {
                        warn!("Failed to read packet: {}", err);
                    }
                }
            }
        }
        debug!("RX handler terminated");
    }

    // listens for tx events and writes the parsed packet to the devices output stream
    fn run_tx_handler(
        device_info: DeviceInfo,
        receiver: Receiver<RodeCasterDeviceIOEvent>,
        canceled: Arc<AtomicBool>
    ) {
        let transport = match RodeCasterHidTransport::new(
            device_info,
            0x03,
            0x04
        ) {
            Ok(transport) => transport,
            Err(err) => {
                error!("Failed to create HID transport: {}", err);
                return;
            }
        };

        while !canceled.load(Ordering::Relaxed) {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(api_event) => match api_event {
                    RodeCasterDeviceIOEvent::PacketReceived(packet) => match transport.write_timeout(&packet, Duration::from_millis(100)) {
                        Ok(_) => { /* packet send successfully */ }
                        Err(e) => {
                            warn!("Failed to write packet: {}", e);
                            // todo: send io error event
                        }
                    },
                    _ => {
                        // do nothing
                    }
                },
                Err(err) => match err {
                    RecvTimeoutError::Disconnected => {
                        trace!("API channel disconnected");
                        break;
                    }
                    RecvTimeoutError::Timeout => {
                        // just a normal timeout, nothing to do here
                    }
                }
            }
        }
        debug!("TX handler terminated");
    }
}