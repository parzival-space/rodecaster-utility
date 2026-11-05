pub(crate) mod common;
mod generic;
pub mod handle;

use std::ffi::CString;
use std::io::Error;
use std::sync::Arc;
use hidapi::{HidApi, HidError};
use crate::device::common::RodeCasterDeviceKind;
use crate::device::generic::RodeCasterDevice;
use crate::device::handle::RodeCasterDeviceHandle;

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
pub async fn open_device(path: CString) -> Result<Arc<Box<dyn RodeCasterDeviceHandle>>, HidError> {
    let hid_api = HidApi::new()?;
    let handle = hid_api.open_path(&path)?;

    match handle.get_device_info()?.product_id() {
        pid if RodeCasterProIIKind::PID.contains(&pid) => {
            // RodeCasterProII::connect(handle);
            todo!("Handle connect device")
        },
        _ => todo!("Handle unsupported device error")
    }
}