pub mod device_report;
pub mod handshake;
pub mod update;

use std::fmt::Debug;
use crate::protocol::{Deserialize, Serialize};

pub(crate) trait Packet: Serialize + Deserialize + Debug + Send {
    // fn get_packet_id(&self) -> u8;
}