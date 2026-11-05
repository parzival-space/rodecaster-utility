use crate::device::common::{RodeCasterDeviceInfo, RodeCasterDeviceInternal, RodeCasterDeviceKind};
use crate::device::handle::RodeCasterDeviceHandle;
use hidapi::{DeviceInfo, HidDevice, HidError};
use std::marker::PhantomData;
use std::panic::RefUnwindSafe;
use std::sync::Arc;

/// A generic device implementation for a RodeCaster protocol compatible device.
#[derive(Debug)]
pub(crate) struct RodeCasterDevice<K: RodeCasterDeviceKind> {
    pid: u16,
    serial: String,
    firmware_version: String,

    // todo: sender

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
        todo!()
    }
}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceHandle for RodeCasterDevice<K> {}