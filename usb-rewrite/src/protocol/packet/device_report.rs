use std::collections::HashMap;
use std::io::Write;
use byteorder::WriteBytesExt;
use nom::bytes::streaming::tag;
use nom::IResult;
use crate::protocol::{Deserialize, Serialize};
use crate::protocol::packet::Packet;
use crate::protocol::types::node::Node;
use crate::protocol::types::value::Value;

#[derive(Debug)]
pub(crate) struct DeviceReportPacket {
    pub report: Node,
}

impl DeviceReportPacket {
    const PACKET_ID: u8 = 2u8;
    pub fn get_children(&self) -> Vec<Node> {
        self.report.children.clone()
    }

    pub fn get_properties(&self) -> HashMap<String, Value> {
        self.report.properties.clone()
    }
}

impl Packet for DeviceReportPacket {}

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