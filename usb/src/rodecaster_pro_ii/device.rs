use crate::common::device::AttachableUsbDevice;
use crate::rodecaster_pro_ii::{PID_RODECASTER_PRO_II, PID_RODECASTER_PRO_II_EXTENDED, PID_RODECASTER_PRO_II_EXTENDED_INPUT, PID_RODECASTER_PRO_II_EXTENDED_OUTPUT};
use crate::VID_RODE;
use anyhow::{anyhow, bail, Result};
use rusb::{Device, DeviceDescriptor, DeviceHandle, GlobalContext, Language, UsbContext};
use std::time::Duration;

const PIDS_RODECASTER_RRO_II: [u16; 4]  = [
    PID_RODECASTER_PRO_II,
    PID_RODECASTER_PRO_II_EXTENDED,
    PID_RODECASTER_PRO_II_EXTENDED_INPUT,
    PID_RODECASTER_PRO_II_EXTENDED_OUTPUT
];

// represents a rodecaster pro ii device that can be connected to using libusb.
// this is currently the only supported use case.
pub struct RodeCasterProIIDevice {
    handle: DeviceHandle<GlobalContext>,
    device: Device<GlobalContext>,
    device_descriptor: DeviceDescriptor,
    timeout: Duration,
    language: Language
}

impl AttachableUsbDevice for RodeCasterProIIDevice {
    fn from_address(bus_number: u8, address: u8) -> Result<Self> {
        let context = GlobalContext::default();
        let devices = context.devices()?;
        let timeout = Duration::from_secs(1);

        // try reading the device at the given address and bus number
        let Some(device) = devices.iter().find(
            |device| device.bus_number() == bus_number && device.address() == address) else {
            bail!("No device found at bus {} and address {}", bus_number, address);
        };

        let Ok(device_descriptor) = device.device_descriptor() else {
            bail!("Failed to read device descriptor, this might not be a RODECaster Pro II");
        };

        // ensure the detected device is actually a RODECaster Pro II by checking the vendor and product IDs
        if VID_RODE != device_descriptor.vendor_id() || !PIDS_RODECASTER_RRO_II.contains(&device_descriptor.product_id()) {
            bail!("The device at bus {} and address {} is not a RODECaster Pro II, unexpected vendor or product ID: {:04x}:{:04x}",
                bus_number, address, device_descriptor.vendor_id(), device_descriptor.product_id());
        };

        let handle = device.open()?;
        let device = handle.device(); // improves reliability of the device reference
        let language = handle.read_languages(timeout)?
            .first()
            .ok_or_else(|| anyhow!("Failed to read supported languages from device, maybe it's not a RODECaster Pro II?"))?
            .to_owned();

        Ok(RodeCasterProIIDevice { handle, device, device_descriptor, timeout, language })
    }

    fn get_bus_number(&self) -> u8 {
        self.device.bus_number()
    }

    fn get_address(&self) -> u8 {
        self.device.address()
    }

    fn get_vendor_id(&self) -> u16 {
        self.device_descriptor.vendor_id()
    }

    fn get_product_id(&self) -> u16 {
        self.device_descriptor.product_id()
    }
}