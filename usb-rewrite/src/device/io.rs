use crate::protocol::hid_transport::{RodeCasterHidTransport, RodeCasterHidTransportError};
use flume::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use hidapi::DeviceInfo;
use log::{debug, error, trace, warn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub(crate) enum RodeCasterDeviceIOEvent {
    /// a raw packet was received from the device.
    /// the packet will only contain the payload bytes without the 4 byte length prefix.
    PacketReceived(Vec<u8>),

    TXError(RodeCasterHidTransportError),
    RXError(RodeCasterHidTransportError),
}

pub(crate) enum RodeCasterDeviceIOCommand {
    /// sends a raw packet payload to the device.
    /// the packet will be prefixed with a 4 byte length prefix.
    SendPacket(Vec<u8>),
}

pub(crate) trait RodeCasterDeviceIOHandler {
    async fn spawn_io_handler(
        device_info: DeviceInfo,
        event_sender: Sender<RodeCasterDeviceIOEvent>,
        command_receiver: Receiver<RodeCasterDeviceIOCommand>,
        cancel: Arc<AtomicBool>
    ) {
        let rx_info = device_info.clone();
        let rx_cancel = Arc::clone(&cancel);
        let tx_info = device_info.clone();
        let tx_cancel = Arc::clone(&cancel);
        let tx_event_sender = event_sender.clone();

        #[cfg(feature = "tokio")]
        {
            use tokio::runtime::Handle;
            if let Ok(handle) = Handle::try_current() {
                debug!("Starting IO background tasks");
                handle.spawn_blocking(move || Self::run_rx_handler(rx_info, event_sender, rx_cancel));
                handle.spawn_blocking(move || Self::run_tx_handler(tx_info, command_receiver, tx_event_sender, tx_cancel));
                return;
            }
        }

        // try to find name in this order: product string > serial number > device path > "unknown"
        let device_name = device_info.product_string().unwrap_or(
            device_info.serial_number().unwrap_or(
                device_info.path().to_str().unwrap_or("rodecaster")));

        use std::thread::Builder;
        let _ = Builder::new()
            .name(format!("{}-rx", device_name))
            .spawn(move || Self::run_rx_handler(rx_info, event_sender, rx_cancel));
        let _ = Builder::new()
            .name(format!("{}-tx", device_name))
            .spawn(move || Self::run_tx_handler(tx_info, command_receiver, tx_event_sender, tx_cancel));
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
            match transport.read_timeout(Duration::from_millis(10)) {
                Ok(packet) => {
                    trace!("Received packet from HID device: {:?}", packet);
                    let _ = sender.send(RodeCasterDeviceIOEvent::PacketReceived(packet));
                }
                Err(err) => match err {
                    RodeCasterHidTransportError::TimeoutError => { /* do nothing */ }
                    err => {
                        warn!("Failed to read packet: {}", err);
                        let _ = sender.send(RodeCasterDeviceIOEvent::RXError(err));
                    }
                }
            }
        }
        debug!("RX handler terminated");
    }

    // listens for tx events and writes the parsed packet to the devices output stream
    fn run_tx_handler(
        device_info: DeviceInfo,
        receiver: Receiver<RodeCasterDeviceIOCommand>,
        sender: Sender<RodeCasterDeviceIOEvent>,
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
                Ok(command) => match command {
                    RodeCasterDeviceIOCommand::SendPacket(packet) => {
                        match transport.write_timeout(&packet, Duration::from_millis(10)) {
                            Ok(_) => { /* packet send successfully */ }
                            Err(err) => {
                                warn!("Failed to write packet: {}", err);
                                let _ = sender.send(RodeCasterDeviceIOEvent::TXError(err));
                            }
                        }
                    }
                }
                Err(err) => match err {
                    RecvTimeoutError::Timeout => { /* do nothing */ }
                    RecvTimeoutError::Disconnected => {
                        trace!("API channel disconnected");
                        break;
                    }
                }
            }
        }

        debug!("TX handler terminated");
    }
}