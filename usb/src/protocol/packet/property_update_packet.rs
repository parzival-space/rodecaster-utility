use crate::protocol::packet::RodeCasterPacket;
use crate::protocol::types::{parse_c_string, write_c_string, StreamableType, Value};
use anyhow::{bail, Result};
use byteorder::WriteBytesExt;
use nom::bytes::streaming::tag;
use nom::number::streaming::{le_u16, le_u8};
use nom::IResult;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PropertyUpdatePacket {
    pub indices: Vec<usize>,
    pub name: String,
    pub value: Value,
}

impl RodeCasterPacket for PropertyUpdatePacket {
    fn from_bytes(input: &[u8]) -> IResult<&[u8], Self> {
        // packet id is 0x01
        let (input, _packet_id) = tag([0x01].as_ref())(input)?;

        // first byte seems to be always 0x01. it's very likely a length indicator for the next
        // indices_count byte. until we have found an actual packet using something different here
        // we will just assume that the first byte is always 0x01.
        let (input, _indices_count_length) = tag([0x01].as_ref())(input)?;

        let (input, indices_count) = le_u8(input)?;
        let (input, indices) = Self::parse_indices(input, indices_count as usize)?;

        let (input, name) = parse_c_string(input)?;
        let (input, value) = Value::parse_from_stream(input)?;

        Ok((
            input,
            Self {
                indices,
                name,
                value,
            },
        ))
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![];

        // packet id is 0x01
        bytes.write_u8(0x01)?;

        // see method above
        bytes.write_u8(0x01)?; // indices count length

        bytes.write_u8(self.indices.len() as u8)?;
        Self::write_indices(&mut bytes, &self.indices)?;

        write_c_string(&mut bytes, &self.name)?;
        self.value.write_to_stream(&mut bytes)?;

        Ok(bytes)
    }
}

impl PropertyUpdatePacket {
    /// Helper function to read the indices list of a PropertyUpdatePacket
    fn parse_indices(input: &[u8], indices_count: usize) -> IResult<&[u8], Vec<usize>> {
        let mut indices = Vec::with_capacity(indices_count as usize);

        let mut last_input = input;
        for _ in 0..indices_count {
            let (input, index_length) = le_u8(last_input)?;
            let (input, index) = match index_length {
                0x00 => (input, 0), // no index value follows, just zero
                0x01 => le_u8(input).map(|res| (res.0, res.1 as usize))?,
                0x02 => le_u16(input).map(|res| (res.0, res.1 as usize))?,
                _ => Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )))?,
            };
            last_input = input;
            indices.push(index);
        }

        Ok((last_input, indices))
    }

    /// Helper function to write the indices list of a PropertyUpdatePacket
    fn write_indices(stream: &mut Vec<u8>, indices: &Vec<usize>) -> Result<()> {
        for index in indices {
            if *index == 0x00 {
                // if the index value is just 0, we can just write a 0x00 byte
                stream.write_u8(0x00)?;
                continue;
            }

            // index written as u8
            if index <= &(u8::MAX as usize) {
                stream.write_u8(0x01)?; // index type u8
                stream.write_u8(index.clone() as u8)?;
            }
            // index written as u16
            else if index <= &(u16::MAX as usize) {
                stream.write_u8(0x02)?; // index type u16
                stream.write_u16::<byteorder::LittleEndian>(index.clone() as u16)?;
            }
            // unsupported
            else {
                bail!(
                    "Index value {} is too large to be written in the current protocol implementation.",
                    index
                );
            }
        }

        Ok(())
    }
}
