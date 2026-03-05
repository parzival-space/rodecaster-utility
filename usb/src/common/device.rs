use anyhow::Result;
use hidapi::{DeviceInfo, HidApi};

pub trait AttachableUsbDevice {
    fn open(api: &mut HidApi, serial: &str) -> Result<Self> where Self: Sized;

    /// Checks if the give device [`DeviceInfo`] is supported by this implementation.
    fn is_device_supported(device: &DeviceInfo) -> bool where Self: Sized;

    fn get_vendor_id(&self) -> u16;
    fn get_product_id(&self) -> u16;

    fn get_manufacturer_string(&self) -> Option<&str>;
    fn get_product_string(&self) -> Option<&str>;

    fn get_serial_number_string(&self) -> Option<&str>;
}

pub trait ExecutableUsbDevice: AttachableUsbDevice {
    fn write(&mut self, data: &[u8]) -> Result<()>;
    fn read(&mut self) -> Result<Vec<u8>>;
}