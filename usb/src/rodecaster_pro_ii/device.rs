use crate::common::device::{AttachableUsbDevice, ExecutableUsbDevice};
use crate::rodecaster_pro_ii::{PID_RODECASTER_PRO_II, PID_RODECASTER_PRO_II_EXTENDED, PID_RODECASTER_PRO_II_EXTENDED_INPUT, PID_RODECASTER_PRO_II_EXTENDED_OUTPUT};
use crate::VID_RODE;
use anyhow::{anyhow, bail, Result};
use rusb::{Device, DeviceDescriptor, DeviceHandle, GlobalContext, Language, UsbContext};
use std::time::Duration;
use log::{debug, warn};
use crate::rodecaster_pro_ii::command::RodeCasterProIIExecutable;

const PIDS_RODECASTER_RRO_II: [u16; 4]  = [
    PID_RODECASTER_PRO_II,
    PID_RODECASTER_PRO_II_EXTENDED,
    PID_RODECASTER_PRO_II_EXTENDED_INPUT,
    PID_RODECASTER_PRO_II_EXTENDED_OUTPUT
];

// todo: not sure if this is the same interface for all PIDs
const DEVICE_INTERFACE: u8 = 0x09;
const DEVICE_ENDPOINT_OUT: u8 = 0x05;
const DEVICE_ENDPOINT_IN: u8 = 0x85;

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

        if !Self::is_supported(bus_number, address) {
            bail!("The device at bus {} and address {} is not a RODECaster Pro II", bus_number, address);
        }

        // try reading the device at the given address and bus number
        let Some(device) = devices.iter().find(
            |device| device.bus_number() == bus_number && device.address() == address) else {
            bail!("No device found at bus {} and address {}", bus_number, address);
        };

        let Ok(device_descriptor) = device.device_descriptor() else {
            bail!("Failed to read device descriptor, this might not be a RODECaster Pro II");
        };

        let handle = device.open()?;
        let device = handle.device(); // improves reliability of the device reference
        let language = handle.read_languages(timeout)?
            .first()
            .ok_or_else(|| anyhow!("Failed to read supported languages from device, maybe it's not a RODECaster Pro II?"))?
            .to_owned();

        // try to claim the device interface
        handle.set_auto_detach_kernel_driver(true)?;
        if handle.claim_interface(DEVICE_INTERFACE).is_err() {
            bail!("Failed to claim Device")
        }
        if (handle.set_alternate_setting(DEVICE_INTERFACE, 0x00).is_err()) {
            warn!("Failed to set alternate setting for device interface.");
        }
        if (handle.clear_halt(DEVICE_ENDPOINT_IN).is_err() || handle.clear_halt(DEVICE_ENDPOINT_OUT).is_err()) {
            warn!("Failed to clear halt on device endpoints, maybe the device is not responding?");
        }

        let mut rodecaster_device = RodeCasterProIIDevice { handle, device, device_descriptor, timeout, language };

        debug!("Successfully connected to RODECaster Pro II. Initializing device...");
        rodecaster_device.request_device_status()?;

        Ok(rodecaster_device)
    }

    fn is_supported(bus_number: u8, address: u8) -> bool {
        let context = GlobalContext::default();
        let Ok(devices) = context.devices() else {
            return false;
        };
        
        let Some(device) = devices.iter().find(
            |device| device.bus_number() == bus_number && device.address() == address) else {
            return false;
        };

        let Ok(device_descriptor) = device.device_descriptor() else {
            return false;
        };

        // ensure the detected device is actually a RODECaster Pro II by checking the vendor and product IDs
        VID_RODE == device_descriptor.vendor_id() && PIDS_RODECASTER_RRO_II.contains(&device_descriptor.product_id())
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

    fn get_manufacturer_string(&self) -> Result<String> {
        Ok(
            self.handle.read_manufacturer_string(
                self.language,
                &self.device_descriptor,
                Duration::from_millis(100)
            )?
        )
    }

    fn get_product_string(&self) -> Result<String> {
        Ok(
            self.handle.read_product_string(
                self.language,
                &self.device_descriptor,
                Duration::from_millis(100)
            )?
        )
    }

    fn get_serial_number_string(&self) -> Result<String> {
        Ok(
            self.handle.read_serial_number_string(
                self.language,
                &self.device_descriptor,
                Duration::from_millis(100)
            )?
        )
    }
}

impl ExecutableUsbDevice for RodeCasterProIIDevice {
    fn write_interrupt(&mut self, data: &[u8]) -> Result<()> {
        if data.len() > 256 {
            bail!("Data is too large. Maximum message size is 256 bytes.");
        }

        self.handle.write_interrupt(
            DEVICE_ENDPOINT_OUT,
            &data,
            self.timeout,
        )?;

        Ok(())
    }

    fn read_interrupt(&mut self) -> Result<Vec<u8>> {
        // messages have a max size of 256 bytes
        let mut buffer = vec![0u8; 256];

        self.handle.read_interrupt(
            DEVICE_ENDPOINT_IN,
            &mut buffer,
            self.timeout
        )?;

        Ok(buffer)
    }
}