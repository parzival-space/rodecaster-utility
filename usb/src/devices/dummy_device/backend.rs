use crate::device::manager::DeviceIdentifier;
use crate::devices::backend::DeviceBackend;
use crate::devices::dummy_device::handle::DummyDeviceHandle;
use crate::error::UsbError;

pub struct DummyDeviceBackend;

impl DeviceBackend for DummyDeviceBackend {
    type Handle = DummyDeviceHandle;

    fn open(identifier: DeviceIdentifier) -> Result<Self::Handle, UsbError> {
        DummyDeviceHandle::open(identifier)
    }
}