use std::io::Write;
use byteorder::WriteBytesExt;
use nom::bytes::streaming::tag;
use nom::IResult;
use crate::protocol::packet::Packet;
use crate::protocol::{Deserialize, Serialize};
use crate::protocol::types::node::Node;

pub(crate) struct DeviceReportPacket {
    report: Node,
}

impl Packet for DeviceReportPacket {
    const PACKET_ID: u8 = 2u8;
}

impl Serialize for DeviceReportPacket {
    fn serialize(&self) -> std::io::Result<Vec<u8>>
    where
        Self: Sized
    {
        let mut stream = vec![];
        stream.write_u8(Self::PACKET_ID)?;
        stream.write_all(self.report.serialize()?.as_ref())?;
        Ok(stream)
    }
}

impl Deserialize for DeviceReportPacket {
    fn deserialize(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, _tag) = tag([Self::PACKET_ID].as_ref())(data)?;
        let (data, report) = Node::deserialize(data)?;
        Ok((data, Self { report }))
    }
}