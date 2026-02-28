use crate::common::device::AttachableUsbDevice;
use crate::rodecaster_pro_ii::{RodeCasterProII, RodeCasterProIIDevice};

/// Represents different types of RODE devices from the RODECaster line.
pub enum RodeCasterDevice {
    RodeCasterProII(Box<dyn RodeCasterProII>)
}

/// Manages USB devices that can be attached to the computer.
/// The actual device implementations are in the `rodecaster_pro_ii` module, but this module can be extended in the future to support other types of devices as well.
pub struct DeviceManager;
impl DeviceManager {
    pub fn open_rodecaster_device(bus_number: u8, address: u8) -> anyhow::Result<RodeCasterDevice> {
        if RodeCasterProIIDevice::is_supported(bus_number, address) {
            let device = RodeCasterProIIDevice::from_address(bus_number, address)?;
            Ok(RodeCasterDevice::RodeCasterProII(Box::new(device)))
        } else {
            anyhow::bail!("No supported device found at bus {} and address {}", bus_number, address);
        }
    }
}