pub mod rodecaster_pro_ii;

pub use nusb;

// Vendor ID used by RODE Microphones
pub const VID_RODE: u16 = 0x19f7;

// Product IDs for RODECASTER PRO II
// The devices reports different product IDs based on the mode it is in.
pub const PID_RODECASTER_PRO_II_EXTENDED: u16 = 0x0072;
pub const PID_RODECASTER_PRO_II_EXTENDED_INPUT: u16 = 0x0078;
pub const PID_RODECASTER_PRO_II_EXTENDED_OUTPUT: u16 = 0x0030;
pub const PID_RODECASTER_PRO_II: u16 = 0x0037;