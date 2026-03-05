use std::ptr::null;
use hidapi::{DeviceInfo, HidApi};
use anyhow::Result;
use crate::common::device::AttachableUsbDevice;
use crate::rodecaster_pro_ii::{RodeCasterProII, RodeCasterProIIDevice};
use crate::VID_RODE;

/// Represents different types of RODE devices from the RODECaster line.
pub enum RodeCasterDevice {
    RodeCasterProII(Box<dyn RodeCasterProII>)
}

/// Manages USB devices that can be attached to the computer.
/// The actual device implementations are in the `rodecaster_pro_ii` module, but this module can be extended in the future to support other types of devices as well.
pub struct DeviceManager {
    hid_api: HidApi,
}
impl DeviceManager {
    pub fn new() -> Result<Self> {
        Ok(DeviceManager {
            hid_api: HidApi::new()?
        })
    }

    pub fn list_devices(&mut self) -> Result<Vec<String>> {
        self.hid_api.refresh_devices()?;

        Ok(
            self.hid_api.device_list()
                .filter(|device| device.vendor_id() == VID_RODE) // Only list RODE devices
                .filter(|device| device.serial_number().is_some())
                .filter_map(|device| {
                    match device {
                        device if RodeCasterProIIDevice::is_device_supported(device) => {
                            Some(device.serial_number().unwrap().to_owned())
                        }
                        _ => {
                            None
                        }
                    }
                })
                .collect::<Vec<String>>()
        )
    }
    
    pub fn open_rodecaster_device(&mut self, serial: &str) -> Result<RodeCasterDevice> {
        self.hid_api.refresh_devices()?;

        Ok(RodeCasterDevice::RodeCasterProII(
            Box::new(RodeCasterProIIDevice::open(&mut self.hid_api, serial)?)
        ))
    }
}