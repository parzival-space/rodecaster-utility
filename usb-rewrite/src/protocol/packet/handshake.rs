use nom::bytes::streaming::tag;
use nom::IResult;
use crate::protocol::{Deserialize, Serialize};
use crate::protocol::packet::Packet;

#[derive(Default, Debug)]
pub(crate) struct HandshakePacket;

impl HandshakePacket {
    // the handshake packet is just a sequence of specific bytes that are hardcoded into the firmwares main GUI application
    const MAGIC_SEQUENCE: [u8; 4] = [0xAD, 0x10, 0xA7, 0xB0];
}

impl Packet for HandshakePacket {}

impl Serialize for HandshakePacket {
    fn serialize(&self) -> std::io::Result<Vec<u8>>
    where
        Self: Sized
    {
        Ok(Self::MAGIC_SEQUENCE.to_vec())
    }
}

impl Deserialize for HandshakePacket {
    fn deserialize(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, _) = tag(Self::MAGIC_SEQUENCE.as_slice())(data)?;
        Ok((data, Self {}))
    }
}