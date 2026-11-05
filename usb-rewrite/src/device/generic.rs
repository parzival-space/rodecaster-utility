use crate::device::common::{RodeCasterDeviceInfo, RodeCasterDeviceInternal, RodeCasterDeviceKind};
use crate::device::handle::{RodeCasterDeviceHandle, RodeCasterDeviceHandleTransport};
use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};
use std::marker::PhantomData;
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use flume::{Receiver, Sender};

/// A generic device implementation for a RodeCaster protocol compatible devices.
#[derive(Debug)]
pub(crate) struct RodeCasterDevice<K: RodeCasterDeviceKind> {
    pid: u16,
    serial: String,
    firmware_version: String,

    io_sender: Option<Sender<()>>,
    io_receiver: Option<Receiver<()>>,

    _kind: PhantomData<K>,
}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceInfo for RodeCasterDevice<K> {
    fn get_product_id(&self) -> u16 {
        self.pid
    }

    fn get_serial(&self) -> String {
        self.serial.clone()
    }

    fn get_version(&self) -> String {
        self.firmware_version.clone()
    }
}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceInternal for RodeCasterDevice<K> {
    async fn connect(device: DeviceInfo) -> Result<Arc<Box<dyn RodeCasterDeviceHandle>>, HidError> {
        let (io_rx_receiver, io_tx_sender) = Self::spawn_event_handler(device.clone()).await;

        // todo: init poll tasks, spawn background worker/thread
        // todo: auto close threads

        Ok(Arc::new(Box::new(RodeCasterDevice {
            pid: device.product_id(),
            serial: device.serial_number().unwrap_or_default().to_string(),
            firmware_version: String::new(),
            io_sender: Some(io_tx_sender),
            io_receiver: Some(io_rx_receiver),
            _kind: PhantomData::<K>,
        })))
    }
}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceHandle for RodeCasterDevice<K> {}
impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceHandleTransport for RodeCasterDevice<K> {}