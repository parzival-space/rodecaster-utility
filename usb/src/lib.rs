mod device_manager;
mod common;
pub mod rodecaster_pro_ii;

/// USB Vendor ID for RODE Devices.
/// Device specific IDs are defined in the respective device module.
pub const VID_RODE: u16 = 0x19f7;

// reexport device manager
pub use device_manager::DeviceManager;
pub use device_manager::RodeCasterDevice;
