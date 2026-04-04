use anyhow::Result;
use hidapi::{DeviceInfo};
use crate::DeviceIdentifier;

pub trait AttachableUsbDevice {
    fn open(device_identifier: DeviceIdentifier) -> Result<Self> where Self: Sized;
    fn get_device_info(&self) -> DeviceInfo;
}