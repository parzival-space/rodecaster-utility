use std::io::Write;
use nom::bytes::streaming::tag;
use nom::IResult;
use crate::protocol::Parseable;
use crate::protocol::types::composite::Composite;

const PACKET_ID: u8 = 2u8;

#[derive(Clone, Debug)]
pub(crate) struct DeviceStatusPacket {
    status: Composite
}

impl DeviceStatusPacket {
    pub fn get_status(&self) -> &Composite {
        &self.status
    }
}

impl Parseable for DeviceStatusPacket {
    fn can_parse(data: &[u8]) -> bool {
        // the first byte has to be 0x02 for a full device report
        // the following bytes are the Composite object containing the status report
        data.len() >= 1 && data[0] == PACKET_ID &&
            Composite::can_parse(&data[1..])
    }

    fn parse(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, _tag) = tag([PACKET_ID].as_ref())(data)?;

        let (data, status) = Composite::parse(data)?;
        Ok((data, Self { status }))
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[PACKET_ID])?;

        self.status.serialize(writer)
    }
}