pub(crate) mod common;
mod generic;
pub mod handle;
mod transport;

use crate::common::DeviceIdentifier;
use crate::device::common::{RodeCasterDeviceInternal, RodeCasterDeviceKind};
use crate::device::generic::{RodeCasterDevice, RodeCasterDeviceHandle};
use hidapi::HidError;
use std::sync::Arc;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum RodeCasterDeviceType {
    #[default]
    RodecasterProII,
    Unknown
}

/// RodeCaster Pro II
#[allow(private_interfaces)]
pub type RodeCasterProII = RodeCasterDevice<RodeCasterProIIKind>;
pub struct RodeCasterProIIKind;
impl RodeCasterDeviceKind for RodeCasterProIIKind {
    const PID: &[u16] = &[
        // firmware 1.6.4
        0x0037, 0x0072, 0x0078, 0x0030,
        // firmware 1.7.3
        0x0094, 0x0092
    ];
    const TYPE: RodeCasterDeviceType = RodeCasterDeviceType::RodecasterProII;
}

/// Opens a RodeCaster device
pub async fn open_device(identifier: DeviceIdentifier) -> Result<Arc<Box<dyn RodeCasterDeviceHandle>>, HidError> {
    match identifier.device_info.product_id() {
        pid if RodeCasterProIIKind::PID.contains(&pid) => {
            RodeCasterProII::connect(identifier.device_info).await
        },
        _ => todo!("Handle unsupported device error")
    }
}