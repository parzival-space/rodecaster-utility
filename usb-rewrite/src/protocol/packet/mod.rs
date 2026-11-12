pub mod device_report;
pub mod handshake;
pub mod update;

use crate::protocol::{Deserialize, Serialize};

pub(crate) trait Packet: Serialize + Deserialize {
    const PACKET_ID: u8;
}