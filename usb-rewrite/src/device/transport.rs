use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use flume::{Receiver, RecvTimeoutError, Sender};
use hidapi::{DeviceInfo, HidApi, HidDevice, HidError, HidResult};
use log::{debug, error, warn};

pub(crate) enum RodeCasterTransportEvent {
    Message(Vec<u8>),
    ThreadQuit
}

pub(crate) trait RodeCasterDeviceHidTransport {
    async fn spawn_io_handler(
        device_info: DeviceInfo,
        rx_sender: Sender<RodeCasterTransportEvent>,
        tx_receiver: Receiver<RodeCasterTransportEvent>,
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
                handle.spawn_blocking(move || Self::run_rx_handler(&rx_info, rx_sender, rx_cancel));
                handle.spawn_blocking(move || Self::run_tx_handler(&tx_info, tx_receiver, tx_cancel));
                return;
            }
        }

        // try to find name in this order: product string > serial number > device path > "unknown"
        let device_name = device_info.product_string().unwrap_or(device_info.serial_number().unwrap_or(device_info.path().to_str().unwrap_or("unknown")));

        use std::thread::Builder;
        let _ = Builder::new()
            .name(format!("{}-rx", device_name))
            .spawn(move || Self::run_rx_handler(&rx_info, rx_sender, rx_cancel));
        let _ = Builder::new()
            .name(format!("{}-tx", device_name))
            .spawn(move || Self::run_tx_handler(&tx_info, tx_receiver, tx_cancel));
    }

    // reads the device input and issues internal events that the protocol implementation can parse
    fn run_rx_handler(
        device_info: &DeviceInfo,
        sender: Sender<RodeCasterTransportEvent>,
        canceled: Arc<AtomicBool>
    ) {
        let handle = match (|| {
            let hid_api = HidApi::new()?;
            device_info.open_device(&hid_api)
        })() {
            Ok(handle) => handle,
            Err(err) => {
                error!("Failed to acquire device handle for RX handler: {}", err);
                let _ = sender.send(RodeCasterTransportEvent::ThreadQuit);
                return;
            }
        };

        while !canceled.load(Ordering::Relaxed) {
            let mut buffer = vec![0u8; 256];
            buffer[0] = 0x04; // receive report id

            match handle.read_timeout(&mut buffer, 100) {
                Ok(bytes_read) => match bytes_read{
                    0 => { /* no message was sent to host */ }
                    _ => {
                        // todo: if start of message, read first 4 bytes as length
                        // read until either end of frame or end of length.
                        // if end of frame first, append to stored vec and continue with next message.
                        // once read complete emit event.
                        debug!("Message: {:?}", buffer);
                        let _ = sender.send(RodeCasterTransportEvent::Message(buffer[1..].to_vec()));
                    }
                },
                Err(err) => {
                    warn!("Error reading from device: {}", err);
                }
            }
        }
        debug!("RX handler terminated");
    }

    // listens for tx events and writes the parsed packet to the devices output stream
    fn run_tx_handler(
        device_info: &DeviceInfo,
        receiver: Receiver<RodeCasterTransportEvent>,
        canceled: Arc<AtomicBool>
    ) {
        let handle = match (|| {
            let hid_api = HidApi::new()?;
            device_info.open_device(&hid_api)
        })() {
            Ok(handle) => handle,
            Err(err) => {
                error!("Failed to acquire device handle for TX handler: {}", err);
                return;
            }
        };

        while !canceled.load(Ordering::Relaxed) {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => match event {
                    RodeCasterTransportEvent::Message(buffer) => {
                        let tx_buffer: Vec<u8> = [
                            &[0x03],            // send report id
                            buffer.as_slice()   // remaining packet bytes
                        ].concat();

                        // todo: handle framing
                        match handle.write(tx_buffer.as_slice()) {
                            Ok(_) => {},
                            Err(err) => {
                                warn!("Error writing to device: {}", err);
                            }
                        }
                    }
                    _ => {}
                },
                Err(err) => match err {
                    RecvTimeoutError::Timeout => { /* do nothing */ },
                    RecvTimeoutError::Disconnected => {
                        warn!("TX channel sender disconnected");
                        break;
                    }
                }
            }
        }
        debug!("TX handler terminated");
    }
}

fn open_device(device_info: DeviceInfo) -> Result<HidDevice, HidError> {
    let hid_api = HidApi::new()?;
    device_info.open_device(&hid_api)
}