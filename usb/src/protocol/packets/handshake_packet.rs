use std::io::Write;
use nom::bytes::streaming::tag;
use nom::IResult;
use crate::protocol::Parseable;

const PAYLOAD: &[u8] = &[0xAD, 0x10, 0xA7, 0xB0];

#[derive(Clone, Debug, Default)]
pub(crate) struct HandshakePacket {}

impl Parseable for HandshakePacket {
    fn can_parse(data: &[u8]) -> bool {
        data.len() == PAYLOAD.len() && data == PAYLOAD
    }

    fn parse(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, _tag) = tag(PAYLOAD.as_ref())(data)?;
        Ok((data, Self::default()))
    }

    fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        Ok(writer.write_all(PAYLOAD)?)
    }
}