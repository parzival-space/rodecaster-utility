use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{unbounded, Receiver, Sender};
use hidapi::DeviceInfo;
use log::error;
use crate::device::io::{run_rx_loop, run_tx_loop, IoRxEvent, IoTxEvent};
use crate::device::model::DeviceModel;
use crate::error::UsbError;
use crate::manager::DeviceIdentifier;
use crate::protocol::types::composite::Composite;

#[derive(Debug)]
pub struct DeviceHandle {
    tx_sender: Sender<IoTxEvent>,
    tx_thread: Option<thread::JoinHandle<()>>,
    rx_receiver: Receiver<IoRxEvent>,
    rx_thread: Option<thread::JoinHandle<()>>,
    shutdown_requested: Arc<AtomicCell<bool>>,

    state: Arc<Mutex<Composite>>,
    device_type: DeviceModel,
}

impl DeviceHandle {
    pub fn open(device_identifier: DeviceIdentifier) -> Result<Self, UsbError> {
        let (io_rx_sender, io_rx_receiver) = unbounded();
        let (io_tx_sender, io_tx_receiver) = unbounded();

        let shutdown_requested = Arc::new(AtomicCell::new(false));
        let state = Arc::new(Mutex::new(Composite::default()));

        // rx thread
        let rx_sender_clone = io_rx_sender.clone();
        let rx_device_info = device_identifier.device_info.clone();
        let rx_shutdown_requested = shutdown_requested.clone();
        let rx_thread = thread::Builder::new()
            .name("io-rx-thread".into())
            .spawn(move || {
                if let Err(err) = run_rx_loop(rx_device_info, rx_sender_clone, rx_shutdown_requested) {
                    error!("Device Rx loop failed: {}", err);
                }
            })?;

        // tx thread
        let tx_receiver_clone = io_tx_receiver.clone();
        let tx_device_info = device_identifier.device_info.clone();
        let tx_thread = thread::Builder::new()
            .name("io-tx-thread".into())
            .spawn(move || {
                if let Err(err) = run_tx_loop(tx_device_info, tx_receiver_clone) {
                    error!("Device Tx loop failed: {}", err);
                }
            })?;

        Ok(Self {
            tx_sender: io_tx_sender,
            tx_thread: Some(tx_thread),
            rx_receiver: io_rx_receiver,
            rx_thread: Some(rx_thread),
            shutdown_requested,
            state,
            device_type: device_identifier.device_model,
        })
    }

    pub fn close(&mut self) -> Result<(), UsbError>{
        if self.shutdown_requested.load() {
            // already shutting down
            return Ok(());
        }
        
        self.shutdown_requested.store(true);
        self.tx_sender.send(IoTxEvent::Close).unwrap_or_default();
        if let Some(tx_thread) = self.tx_thread.take() {
            tx_thread.join().unwrap_or_default();
        }
        if let Some(rx_thread) = self.rx_thread.take() {
            rx_thread.join().unwrap_or_default();
        }
        Ok(())
    }
}