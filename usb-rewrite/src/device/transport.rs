use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use flume::{Sender};
use hidapi::{DeviceInfo, HidApi};
use log::{debug, error, warn};

pub(crate) enum RodeCasterTransportEvent {
    Message(Vec<u8>),
    ThreadQuit
}

pub(crate) trait RodeCasterDeviceHidTransport {
    fn run_rx_handler(device_info: &DeviceInfo, sender: Sender<RodeCasterTransportEvent>, canceled: Arc<AtomicBool>) {
        let hid_api = match HidApi::new() {
            Ok(api) => api,
            Err(err) => {
                error!("Error creating HID API: {}", err);
                let _ = sender.send(RodeCasterTransportEvent::ThreadQuit);
                return;
            }
        };

        let handle = match device_info.open_device(&hid_api) {
            Ok(handle) => handle,
            Err(err) => {
                error!("Error opening device: {}", err);
                let _ = sender.send(RodeCasterTransportEvent::ThreadQuit);
                return;
            }
        };

        while !canceled.load(Ordering::Relaxed) {
            let mut buffer = Vec::with_capacity(256);
            buffer.resize_with(256, || 0x00);
            buffer[0] = 0x04; // receive report id

            match handle.read_timeout(&mut buffer, 100) {
                Ok(bytes_read) => match bytes_read{
                    0 => { /* no message was sent to host */ }
                    _ => {
                        // todo: handle framing
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

}
