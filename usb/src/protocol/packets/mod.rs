use nom::IResult;
use crate::protocol::packets::device_status_packet::DeviceStatusPacket;
use crate::protocol::packets::handshake_packet::HandshakePacket;
use crate::protocol::packets::property_patch_packet::PropertyPatchPacket;
use crate::protocol::Parseable;

pub mod device_status_packet;
pub mod property_patch_packet;
pub mod handshake_packet;

pub(crate) enum Packet {
    DeviceStatus(DeviceStatusPacket),
    PropertyPatch(PropertyPatchPacket),
    Handshake(HandshakePacket),
    Unknown(Vec<u8>),
}

pub(crate) fn parse_packet(bytes: &[u8]) -> IResult<&[u8], Packet> {
    match true {
        _ if DeviceStatusPacket::can_parse(bytes) => DeviceStatusPacket::parse(bytes)
            .map(|result| (result.0, Packet::DeviceStatus(result.1))),
        _ if PropertyPatchPacket::can_parse(bytes) => PropertyPatchPacket::parse(bytes)
            .map(|result| (result.0, Packet::PropertyPatch(result.1))),
        _ if HandshakePacket::can_parse(bytes) => HandshakePacket::parse(bytes)
            .map(|result| (result.0, Packet::Handshake(result.1))),
        _ => Ok((&[], Packet::Unknown(bytes.to_vec()))),
    }
}