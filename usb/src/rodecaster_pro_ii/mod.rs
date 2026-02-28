pub mod device;

use crate::common::device::AttachableUsbDevice;
use crate::rodecaster_pro_ii::device::RodeCasterProIIDevice;


// USB product IDs for the RodeCaster Pro II.
// The device reports different IDs based on the multichannel configuration.
pub const PID_RODECASTER_PRO_II_EXTENDED: u16 = 0x0072;
pub const PID_RODECASTER_PRO_II_EXTENDED_INPUT: u16 = 0x0078;
pub const PID_RODECASTER_PRO_II_EXTENDED_OUTPUT: u16 = 0x0030;
pub const PID_RODECASTER_PRO_II: u16 = 0x0037;


pub trait RodeCasterProII: AttachableUsbDevice {}
impl RodeCasterProII for RodeCasterProIIDevice {}

