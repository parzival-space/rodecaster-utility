use std::collections::HashMap;
use nom::bytes::streaming::tag;
use nom::IResult;
use crate::protocol::packet::RodeCasterPacket;
use crate::protocol::types::{StreamableType, Structured};

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceReportPacket {
    pub report: Structured,
}

impl RodeCasterPacket for DeviceReportPacket {
    fn from_bytes(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        // packet id is 0x02
        let (input, _packet_id) = tag([0x02].as_ref())(input)?;

        let (input, root_struct) = Structured::parse_from_stream(input)?;

        Ok((input, DeviceReportPacket { report: root_struct }))
    }

    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = vec![];

        // dummy implementation, structured parsing is not implemented yet
        bytes.push(0x02u8);
        self.report.write_to_stream(&mut bytes)?;

        Ok(bytes)
    }
}