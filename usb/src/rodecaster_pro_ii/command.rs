use anyhow::Result;
use log::info;
use crate::common::device::ExecutableUsbDevice;
use crate::rodecaster_pro_ii::RodeCasterProIIDevice;

impl RodeCasterProIIExecutable for RodeCasterProIIDevice {}

pub trait RodeCasterProIIExecutable: ExecutableUsbDevice {
    /// Requests the device to report its current status, including the device configuration and the
    /// state of all controls.
    /// 
    /// This command needs to be sent <b>at least once</b> after connecting to the device, otherwise
    /// the device won't respond to any other subsequent commands.
    fn request_device_status(&mut self) -> Result<()> {
        info!("Value: {}", 0x69C90100);

        self.write(&[
            0x03, 0x04, 0x00, 0x00, 0x00, 0xAD, 0x10, 0xA7, 0xB0
        ])
    }
}

