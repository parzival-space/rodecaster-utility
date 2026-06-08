pub mod handle;
pub mod io;
pub mod state;
pub mod backend;

use crate::devices::{DeviceRegistryEntry, VID_RODE, DeviceType};

pub(crate) const PID_RODECASTER_PRO_II: &[u16] = &[0x0037, 0x0072, 0x0078, 0x0030, 0x0094, 0x0092];

pub(crate) const REGISTRY_ENTRY: DeviceRegistryEntry = DeviceRegistryEntry {
    vendor_id: VID_RODE,
    product_ids: PID_RODECASTER_PRO_II,
    device_type: DeviceType::RodeCasterProII,
};