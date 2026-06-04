use crate::device::manager::DeviceIdentifier;
use crate::error::UsbError;

#[derive(Debug)]
pub struct DummyDeviceHandle {}

impl DummyDeviceHandle {
    pub fn open(_identifier: DeviceIdentifier) -> Result<Self, UsbError> {
        Ok(Self {})
    }
}