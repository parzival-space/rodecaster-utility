use log::{debug, error};
use rusb::{Device, DeviceDescriptor, DeviceHandle, Error, GlobalContext, Language, UsbContext};
use std::time::Duration;

pub const INTERFACE_CONTROL: u8 = 9;

pub struct RODECaster {
    handle: DeviceHandle<GlobalContext>,
    device: Device<GlobalContext>,
    device_descriptor: DeviceDescriptor,
    timeout: Duration,
    language: Language
}

impl RODECaster {
    pub fn from_handle(handle: DeviceHandle<GlobalContext>, device_descriptor: DeviceDescriptor) -> Result<Self, Error> {
        let device = handle.device();
        let timeout = Duration::from_secs(1);

        debug!("Trying to connect to RODECaster Pro II at {:?}", device);
        let languages = handle.read_languages(timeout)?;
        let language = languages.first().ok_or(Error::NotSupported)?.to_owned();

        debug!("Set active configuration: {:?}", handle.set_active_configuration(1));

        handle.set_auto_detach_kernel_driver(true)?;
        if (handle.claim_interface(INTERFACE_CONTROL).is_err()) {
            error!("Failed to claim interface, maybe it's already claimed by another process? {:?}", handle);
            return Err(Error::NotSupported);
        }

        Ok(Self {
            handle,
            device,
            device_descriptor,
            timeout,
            language
        })
    }
}