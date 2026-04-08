use crate::protocol::types::Structured;
use crate::DeviceIdentifier;
use anyhow::Result;
use hidapi::DeviceInfo;

pub trait AttachableUsbDevice {
    fn open(device_identifier: DeviceIdentifier) -> Result<Self>
    where
        Self: Sized;
    fn get_device_info(&self) -> DeviceInfo;
    fn get_state(&self) -> Result<Structured>; // todo: replace with stronger type definition
}
