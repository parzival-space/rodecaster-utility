use crate::devices::manager::DeviceIdentifier;
use crate::devices::backend::DeviceBackend;
use crate::devices::DeviceType;
use crate::devices::dummy_device::backend::DummyDeviceBackend;
use crate::devices::dummy_device::handle::DummyDeviceHandle;
use crate::devices::rodecaster_pro_ii::backend::RodeCasterProIIBackend;
use crate::devices::rodecaster_pro_ii::handle::RodeCasterProIIHandle;
use crate::error::UsbError;

pub enum DeviceHandle {
    Dummy(DummyDeviceHandle),
    RodeCasterProII(RodeCasterProIIHandle),
}

pub fn open_device(device_identifier: DeviceIdentifier) -> Result<DeviceHandle, UsbError> {
    match device_identifier.device_type {
        DeviceType::DummyDevice => {
            // a simple null device, doesn't do anything. used for testing
            Ok(DeviceHandle::Dummy(DummyDeviceBackend::open(device_identifier)?))
        }
        DeviceType::RodeCasterProII => {
            Ok(DeviceHandle::RodeCasterProII(RodeCasterProIIBackend::open(device_identifier)?))
        }
    }
}