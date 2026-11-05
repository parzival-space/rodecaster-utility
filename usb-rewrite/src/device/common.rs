use crate::device::RodeCasterDeviceType;

/// Representation of a RodeCaster device
pub trait RodeCasterDeviceKind: Send + Sync + 'static {
    const PID: &[u16];
    const TYPE: RodeCasterDeviceType;
}

pub trait RodeCasterDeviceInfo {
    fn get_product_id(&self) -> u16;
    fn get_serial(&self) -> String;
    fn get_version(&self) -> String;
}