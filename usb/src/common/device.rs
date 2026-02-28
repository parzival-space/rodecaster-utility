use anyhow::Result;

pub trait AttachableUsbDevice {
    fn from_address(bus_number: u8, address: u8) -> Result<Self> where Self: Sized;

    fn get_bus_number(&self) -> u8;
    fn get_address(&self) -> u8;

    fn get_vendor_id(&self) -> u16;
    fn get_product_id(&self) -> u16;

    fn get_manufacturer_string(&self) -> Result<String>;
    fn get_product_string(&self) -> Result<String>;

    fn get_serial_number_string(&self) -> Result<String>;
}