pub mod device;

use crate::common::device::{AttachableUsbDevice};

pub use device::RodeCasterProIIDevice;

impl RodeCasterProII for RodeCasterProIIDevice {}
pub trait RodeCasterProII: AttachableUsbDevice {}

