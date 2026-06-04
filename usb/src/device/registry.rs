use crate::devices::DeviceType;
use crate::devices::rodecaster_pro_ii;

pub const VID_RODE: u16 = 0x19F7;

pub struct DeviceRegistryEntry {
    pub vendor_id: u16,
    pub product_ids: &'static [u16],
    pub device_type: DeviceType, // todo: what is this?
}

pub(crate) const KNOWN_DEVICES: &[DeviceRegistryEntry] = &[
    rodecaster_pro_ii::REGISTRY_ENTRY,
];