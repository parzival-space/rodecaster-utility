pub mod device;
pub mod command;

use crate::common::device::{AttachableUsbDevice, ExecutableUsbDevice};

pub use device::RodeCasterProIIDevice;

impl RodeCasterProII for RodeCasterProIIDevice {}
pub trait RodeCasterProII: AttachableUsbDevice + ExecutableUsbDevice {}

