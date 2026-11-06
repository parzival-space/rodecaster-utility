use crate::device::common::{RodeCasterDeviceInfo, RodeCasterDeviceInternal, RodeCasterDeviceKind};
use crate::device::handle::{RodeCasterDeviceHandle};
use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};
use std::marker::PhantomData;
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use flume::{Receiver, Sender};
use log::debug;
use tokio::runtime::Handle;
use crate::device::transport::RodeCasterDeviceHidTransport;

/// A generic device implementation for a RodeCaster protocol compatible devices.
#[derive(Debug)]
pub(crate) struct RodeCasterDevice<K: RodeCasterDeviceKind> {
    pid: u16,
    serial: String,
    firmware_version: String,

    cancel_handler: Arc<AtomicBool>,

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
        let cancel_handler = Arc::new(AtomicBool::new(false));
        let (io_rx_sender, io_rx_receiver) = flume::unbounded();
        let (io_tx_sender, io_tx_receiver) = flume::unbounded();

        Self::spawn_io_handler(device.clone(), io_rx_sender, io_tx_receiver, Arc::clone(&cancel_handler)).await;

        Ok(Arc::new(Box::new(RodeCasterDevice {
            pid: device.product_id(),
            serial: device.serial_number().unwrap_or_default().to_string(),
            firmware_version: String::new(),
            cancel_handler,
            _kind: PhantomData::<K>,
        })))
    }
}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceHandle for RodeCasterDevice<K> {}
impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceHidTransport for RodeCasterDevice<K> {}

impl<K: RodeCasterDeviceKind> Drop for RodeCasterDevice<K> {
    fn drop(&mut self) {
        self.cancel_handler.store(true, Ordering::Release);
    }
}