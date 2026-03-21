use anyhow::Result;
use hidapi::{DeviceInfo};
use crate::DeviceIdentifier;

pub trait AttachableUsbDevice {
    fn open(device_identifier: DeviceIdentifier) -> Result<Self> where Self: Sized;
    fn get_device_info(&self) -> DeviceInfo;
}

pub trait ExecutableUsbDevice: AttachableUsbDevice {
    fn write(&mut self, data: Vec<u8>) -> Result<()>;
    fn read(&mut self) -> Result<Vec<u8>>;
}