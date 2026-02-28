use crate::common::device::AttachableUsbDevice;
use crate::rodecaster_pro_ii::device::RodeCasterProIIDevice;
use crate::rodecaster_pro_ii::RodeCasterProII;
use anyhow::Result;

pub mod rodecaster_pro_ii;
mod common;


/// USB Vendor ID for RODE Devices. Device specific IDs are defined in the respective device module.
pub const VID_RODE: u16 = 0x19f7;


/// Represents different types of RODE devices from the RODECaster line.
pub enum RodeCasterDevice {
    RodeCasterProII(Box<dyn RodeCasterProII>)
}


/// Manages USB devices that can be attached to the computer.
/// The actual device implementations are in the `rodecaster_pro_ii` module, but this module can be extended in the future to support other types of devices as well.
pub struct DeviceManager;
impl DeviceManager {
    pub fn open_rodecaster_device(bus_number: u8, address: u8) -> Result<RodeCasterDevice> {
        return Ok(
            RodeCasterDevice::RodeCasterProII(
                Box::new(
                    RodeCasterProIIDevice::from_address(bus_number, address)?
                )
            )
        );
    }
}