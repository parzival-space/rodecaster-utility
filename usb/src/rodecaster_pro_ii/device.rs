use crate::common::device::{AttachableUsbDevice, ExecutableUsbDevice};
use crate::rodecaster_pro_ii::{PID_RODECASTER_PRO_II, PID_RODECASTER_PRO_II_EXTENDED, PID_RODECASTER_PRO_II_EXTENDED_INPUT, PID_RODECASTER_PRO_II_EXTENDED_OUTPUT};
use crate::VID_RODE;
use anyhow::{anyhow, bail, Result};
use hidapi::{DeviceInfo, HidApi, HidDevice, HidResult};
use std::time::Duration;
use log::{debug, warn};
use crate::rodecaster_pro_ii::command::RodeCasterProIIExecutable;

pub struct RodeCasterProIIDevice {
    device: HidDevice,
    device_info: DeviceInfo,
}

impl AttachableUsbDevice for RodeCasterProIIDevice {
    fn open(api: &mut HidApi, serial: &str) -> Result<Self> {
        let device_info = api.device_list()
            .find(|device|
                VID_RODE.eq(&device.vendor_id()) &&
                    Self::is_device_supported(device) == true &&
                device.serial_number().map_or(false, |s| serial.eq(s)))
            .ok_or_else(|| anyhow!("No RODECaster Pro II found with Serial {}", serial))?;

        let device = match device_info.open_device(&api) {
            Ok(device) => device,
            Err(e) => {
                bail!("Failed to open RODECaster Pro II with Serial {}: {}", serial, e);
            }
        };

        let mut rodecaster_device = RodeCasterProIIDevice { device, device_info: device_info.clone() };

        debug!("Successfully connected to RODECaster Pro II. Initializing device...");
        rodecaster_device.request_device_status()?;

        Ok(rodecaster_device)
    }

    fn is_device_supported(device: &DeviceInfo) -> bool
    where
        Self: Sized
    {
        PID_RODECASTER_PRO_II.eq(&device.product_id()) ||
        PID_RODECASTER_PRO_II_EXTENDED.eq(&device.product_id()) ||
        PID_RODECASTER_PRO_II_EXTENDED_INPUT.eq(&device.product_id()) ||
        PID_RODECASTER_PRO_II_EXTENDED_OUTPUT.eq(&device.product_id())
    }

    fn get_vendor_id(&self) -> u16 {
        self.device_info.vendor_id()
    }

    fn get_product_id(&self) -> u16 {
        self.device_info.product_id()
    }

    fn get_manufacturer_string(&self) -> Option<&str> {
        self.device_info.manufacturer_string()
    }

    fn get_product_string(&self) -> Option<&str> {
        self.device_info.product_string()
    }

    fn get_serial_number_string(&self) -> Option<&str> {
        self.device_info.serial_number()
    }
}

impl ExecutableUsbDevice for RodeCasterProIIDevice {
    fn write(&mut self, data: &[u8]) -> Result<()> {
        if data.len() > 256 {
            bail!("Data is too large. Maximum message size is 256 bytes.");
        }

        match self.device.write(&data) {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(anyhow!("Failed to write message: {}", e))
            }
        }
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        // messages have a max size of 256 bytes
        let mut buffer = vec![0u8; 256];

        match self.device.read(&mut buffer) {
            Ok(_) => Ok(buffer),
            Err(e) => {
                Err(anyhow!("Failed to read message: {}", e))
            }
        }
    }
}