pub mod device;
pub mod command;

use crate::common::device::{AttachableUsbDevice, ExecutableUsbDevice};

// USB product IDs for the RodeCaster Pro II.
// The device reports different IDs based on the multichannel configuration.
pub const PID_RODECASTER_PRO_II: u16 = 0x0037;
pub const PID_RODECASTER_PRO_II_EXTENDED: u16 = 0x0072;
pub const PID_RODECASTER_PRO_II_EXTENDED_INPUT: u16 = 0x0078;
pub const PID_RODECASTER_PRO_II_EXTENDED_OUTPUT: u16 = 0x0030;

// USB product IDs added by the 1.7.3 firmware
pub const PID_RODECASTER_PRO_II_DYNAMIC_EXTENDED: u16 = 0x0094;
pub const PID_RODECASTER_PRO_II_DYNAMIC_EXTENDED_OUTPUT: u16 = 0x0092;

pub use device::RodeCasterProIIDevice;
use crate::rodecaster_pro_ii::command::RodeCasterProIIExecutable;

impl RodeCasterProII for RodeCasterProIIDevice {}
pub trait RodeCasterProII:
    ExecutableUsbDevice +
    RodeCasterProIIExecutable {}

