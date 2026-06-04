pub mod rodecaster_pro_ii;
pub mod dummy_device;
pub mod open;
mod backend;

pub const VID_RODE: u16 = 0x19F7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    DummyDevice,
    RodeCasterProII,
}

pub(crate) struct DeviceRegistryEntry {
    pub vendor_id: u16,
    pub product_ids: &'static [u16],
    pub device_type: DeviceType, // todo: what is this?
}

pub(crate) const KNOWN_DEVICES: &[DeviceRegistryEntry] = &[
    dummy_device::REGISTRY_ENTRY,
    rodecaster_pro_ii::REGISTRY_ENTRY,
];