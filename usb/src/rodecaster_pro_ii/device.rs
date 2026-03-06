use crate::common::device::{AttachableUsbDevice, ExecutableUsbDevice};
use crate::rodecaster_pro_ii::{PID_RODECASTER_PRO_II, PID_RODECASTER_PRO_II_EXTENDED, PID_RODECASTER_PRO_II_EXTENDED_INPUT, PID_RODECASTER_PRO_II_EXTENDED_OUTPUT};
use crate::VID_RODE;
use anyhow::{anyhow, bail, Result};
use hidapi::{DeviceInfo, HidApi, HidDevice, HidResult};
use std::time::Duration;
use log::{debug, warn};
use crate::rodecaster_pro_ii::command::RodeCasterProIIExecutable;

const HID_REPORT_ID_SEND: u8 = 0x03;
const HID_REPORT_ID_RECEIVE: u8 = 0x04;

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

        device.set_blocking_mode(true)?;

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
    fn write(&mut self, data: Vec<u8>) -> Result<()> {
        if data.len() > 255 {
            bail!("Data length exceeds maximum allowed size of 255 bytes");
        }

        // messages has a size of 256 bytes, but the first byte is reserved for the report ID.
        let mut buffer = vec![0u8; 256];
        buffer[0] = HID_REPORT_ID_SEND;
        buffer[1..(data.len() + 1)].copy_from_slice(&data);

        debug!("Writing report with id ({}), data: {:02x?}", buffer[0], buffer[1..].to_vec());
        match self.device.write(&buffer) {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(anyhow!("Failed to write message: {}", e))
            }
        }
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        // messages have a max size of 256 bytes
        let mut buffer = vec![0u8; 256];
        buffer[0] = HID_REPORT_ID_RECEIVE;

        match self.device.read(&mut buffer) {
            Ok(length) => {
                if length != buffer.len() || buffer[0] != HID_REPORT_ID_RECEIVE {
                    bail!("Received message with unexpected length or report ID. Expected length: {}, actual length: {}, expected report ID: {}, actual report ID: {}",
                        buffer.len(), length, HID_REPORT_ID_RECEIVE, buffer[0]);
                }

                // success, return buffer without the first byte
                debug!("Read report with id ({}), data: {:02x?}", buffer[0], buffer[1..].to_vec());
                Ok(buffer[1..].to_vec())
            },
            Err(e) => {
                bail!("Failed to read message: {}", e)
            }
        }
    }
}