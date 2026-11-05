use std::panic::RefUnwindSafe;
use flume::{Receiver, Sender};
use futures::select;
use hidapi::{DeviceInfo, HidApi, HidError};
use log::{debug, warn};
use crate::device::common::RodeCasterDeviceInfo;

pub trait RodeCasterDeviceHandle: RodeCasterDeviceInfo + RefUnwindSafe + Sync + Send {
    // todo: open/coonect - init transport
    // todo: handle message
    // todo: store state
}

pub(crate) trait RodeCasterDeviceHandleTransport {
    async fn spawn_event_handler(
        device_info: DeviceInfo
    ) -> (Receiver<()>, Sender<()>) {
        let (io_rx_sender, io_rx_receiver) = flume::unbounded();
        let (io_tx_sender, io_tx_receiver) = flume::unbounded();

        let rx_device_info = device_info.clone();
        let tx_device_info = device_info.clone();

        #[cfg(feature = "tokio")] {
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn_blocking(move || Self::run_rx_loop(rx_device_info, io_rx_sender));
                handle.spawn_blocking(move || Self::run_tx_loop(tx_device_info, io_tx_receiver));
            }
            return (io_rx_receiver, io_tx_sender);
        }

        use std::thread;
        debug!("Spawning IO background threads");
        let thread_name = device_info.serial_number().unwrap_or(device_info.path().to_str().unwrap_or("unknown"));
        thread::Builder::new()
            .name(format!("{}-rx", thread_name))
            .spawn(move || Self::run_rx_loop(rx_device_info, io_rx_sender))
            .expect("Failed to spawn RX thread");
        thread::Builder::new()
            .name(format!("{}-tx", thread_name))
            .spawn(move || Self::run_tx_loop(tx_device_info, io_tx_receiver))
            .expect("Failed to spawn TX thread");

        (io_rx_receiver, io_tx_sender)
    }

    fn run_rx_loop(device_info: DeviceInfo, sender: Sender<()>) -> Result<(), HidError> {
        let hid_api = HidApi::new()?;
        let handle = device_info.open_device(&hid_api)?;

        let mut buffer = [0u8; 256];
        buffer[0] = 0x04; // receive report id

        loop {
            // todo: allow closing the loop

            match handle.read_timeout(&mut buffer, 500) {
                Ok(bytes_read) => {
                    // todo: parse message
                    debug!("Received {} bytes from device", bytes_read);
                },
                Err(e) => warn!("Failed to read from device: {:?}", e)
            }
        }
    }

    fn run_tx_loop(device_info: DeviceInfo, receiver: Receiver<()>) -> Result<(), HidError> {
        let hid_api = HidApi::new()?;
        let handle = device_info.open_device(&hid_api)?;

        // 0x03 HID_REPORT_ID_SEND

        todo!("Implement chunked writes");

        Ok(())
    }
}