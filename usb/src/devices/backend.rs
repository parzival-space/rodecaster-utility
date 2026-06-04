use crate::devices::manager::DeviceIdentifier;
use crate::error::UsbError;

pub(crate) trait DeviceBackend {
    type Handle;
    fn open(identifier: DeviceIdentifier) -> Result<Self::Handle, UsbError>;
}