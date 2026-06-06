use std::io::Write;
use log::warn;
use nom::bytes::streaming::tag;
use nom::IResult;
use nom::number::streaming::le_u8;
use crate::protocol::Parseable;
use crate::protocol::types::value::Value;

const PACKET_ID: u8 = 1u8;

#[derive(Clone, Debug)]
pub(crate) struct PropertyPatchPacket {
    indices: Vec<usize>,
    name: String,
    value: Value
}

impl PropertyPatchPacket {
    pub fn get_indices(&self) -> &Vec<usize> {
        &self.indices
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_value(&self) -> &Value {
        &self.value
    }
}

impl Parseable for PropertyPatchPacket {
    fn can_parse(data: &[u8]) -> bool {
        // the first byte has to be 0x01 for a full device report
        // at least 2 bytes have to follow then
        // 1 byte with an unknown purpose (most likely bytes count for next byte)
        // 1 byte containing the number of indices
        data.len() >= 3 && data[0] == PACKET_ID
    }

    fn parse(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, _tag) = tag([PACKET_ID].as_ref())(data)?;

        // first data byte seems to be always 0x01. it's very likely a length indicator for the next
        // indices_count byte. until we have found an actual packet using something different here
        // we will just assume that the first byte is always 0x01.
        let (data, unknown1) = le_u8(data)?;
        if unknown1 != 0x01 {
            warn!("Unexpected value for first byte in PropertyPatchPacket: expected 0x01, got {:#04x}. Please report this to an developer!", unknown1);
        }

        let (mut data, indices_count) = le_u8(data)?;
        let mut indices = Vec::with_capacity(indices_count as usize);
        for _ in 0..indices_count {
            let (data_boxed, index_length) = le_u8(data)?;
            let (data_boxed, index) = match index_length {
                0u8 => (data_boxed, 0), // 0 entry
                1u8 => le_u8(data_boxed).map(|res| (res.0, res.1 as usize))?, // 1 byte index
                2u8 => le_u8(data_boxed).map(|res| (res.0, res.1 as usize - 1))?, // 2 byte index
                _ => Err(nom::Err::Failure(
                    nom::error::Error::new(data_boxed, nom::error::ErrorKind::Digit)
                ))? // invalid index length
            };
            data = data_boxed;
            indices.push(index);
        }

        let (data, name) = String::parse(data)?;
        let (data, value) = Value::parse(data)?;

        Ok((data, Self { indices, name, value }))
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        todo!()
    }
}