pub mod backend;
pub mod handle;

use crate::device::registry::DeviceRegistryEntry;

pub const REGISTRY_ENTRY: DeviceRegistryEntry = DeviceRegistryEntry {
    vendor_id: 0xFFFF,
    product_ids: &[0xFFFF],
    device_type: crate::devices::DeviceType::DummyDevice,
};