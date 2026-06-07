pub mod backend;
pub mod handle;

use crate::devices::{DeviceRegistryEntry, DeviceType};

pub(crate) const REGISTRY_ENTRY: DeviceRegistryEntry = DeviceRegistryEntry {
    vendor_id: 0xFFFF,
    product_ids: &[0xFFFF],
    device_type: DeviceType::DummyDevice,
};