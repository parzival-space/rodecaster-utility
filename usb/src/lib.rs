mod device_manager;
mod common;
pub mod rodecaster_pro_ii;
pub mod parser;
mod protocol;

/// USB Vendor ID for RODE Devices.
/// Device specific IDs are defined in the respective device module.
pub const VID_RODE: u16 = 0x19f7;

// re-export device manager
pub use device_manager::DeviceManager;
pub use device_manager::DeviceIdentifier;
pub use device_manager::DeviceType;
pub use device_manager::HotPlugDeviceEvent;
pub use device_manager::OpenDeviceResult;

pub use rodecaster_pro_ii::RodeCasterProII;