use crate::device::manager::DeviceIdentifier;
use crate::devices::backend::DeviceBackend;
use crate::devices::rodecaster_pro_ii::handle::RodeCasterProIIHandle;
use crate::error::UsbError;

pub struct RodeCasterProIIBackend;

impl DeviceBackend for RodeCasterProIIBackend {
    type Handle = RodeCasterProIIHandle;

    fn open(identifier: DeviceIdentifier) -> Result<Self::Handle, UsbError> {
        RodeCasterProIIHandle::open(identifier)
    }
}