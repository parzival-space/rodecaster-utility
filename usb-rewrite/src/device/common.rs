use crate::device::RodeCasterDeviceType;
use crate::device::generic::RodeCasterDeviceHandle;
use hidapi::{DeviceInfo, HidError};
use std::sync::Arc;

/// Representation of a RodeCaster device
pub trait RodeCasterDeviceKind: Send + Sync + 'static {
    const VID: &[u16] = &[0x19f7]; // Vendor ID of RODE
    const PID: &[u16];
    const TYPE: RodeCasterDeviceType;
}

pub trait RodeCasterDeviceInfo {
    fn get_product_id(&self) -> u16;
    fn get_serial(&self) -> String;
    fn get_version(&self) -> String;
}

pub(crate) trait RodeCasterDeviceInternal {
    async fn connect(
        handle: DeviceInfo
    ) -> Result<Arc<Box<dyn RodeCasterDeviceHandle>>, HidError>;
}