use std::io::Write;
use byteorder::{WriteBytesExt, LE};
use nom::bytes::streaming::tag;
use nom::{error_position, IResult};
use nom::error::ErrorKind;
use nom::number::streaming::{le_u16, le_u8};
use crate::protocol::packet::Packet;
use crate::protocol::{Deserialize, Serialize};
use crate::protocol::types::value::Value;

pub(crate) struct UpdatePacket {
    indices: Vec<usize>,
    field: String,
    value: Value,
}

impl UpdatePacket {
    const PACKET_ID: u8 = 1u8;
}

impl Serialize for UpdatePacket {
    fn serialize(&self) -> std::io::Result<Vec<u8>>
    where
        Self: Sized
    {
        let mut stream = vec![];
        stream.write_u8(Self::PACKET_ID)?;
        
        // write indices list length, currently we always assume the length is <255
        stream.write_u8(1u8)?; // 1 byte for length
        stream.write_u8(self.indices.len() as u8)?;
        
        for index in &self.indices {
            match index { 
                _ if *index == 0 => stream.write_u8(0)?,
                _ if *index <= usize::from(u8::MAX) => {
                    stream.write_u8(1u8)?;
                    stream.write_u8(*index as u8)?;
                },
                _ if *index <= usize::from(u16::MAX) => {
                    stream.write_u8(2u8)?;
                    stream.write_u16::<LE>(*index as u16)?;
                }
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData, "invalid index length")
                )?,
            }
        };
        
        stream.write_all(&self.field.serialize()?)?;
        stream.write_all(&self.value.serialize()?)?;
        
        Ok(stream)
    }
}

impl Deserialize for UpdatePacket {
    fn deserialize(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, _id) = tag([Self::PACKET_ID].as_ref())(data)?;
        
        // first byte is probably the indices list length. I haven't yet seen a value that is not 1
        // so we will just assume for now its always 1
        let (data, indices_count_length) = le_u8(data)?;
        let (mut data, indices_count) = match indices_count_length {
            1u8 => le_u8(data).map(|(data, value)| (data, value as usize)),
            _ => {
                // fixme: implement proper length parsing if this happens to be greater than 1
                Err(nom::Err::Error(error_position!(data, ErrorKind::Fix)))
            }
        }?;
        
        let mut indices = Vec::with_capacity(indices_count as usize);
        for _ in 0..indices_count {
            let (data_inner, index_length) = le_u8(data)?;
            let (data_inner, index) = match index_length {
                0u8 => Ok((data_inner, 0)),
                1u8 => le_u8(data_inner).map(|(data_inner, value)| (data, value as usize)),
                2u8 => le_u16(data_inner).map(|(data_inner, value)| (data, value as usize)),
                _ => Err(nom::Err::Error(error_position!(data, ErrorKind::Fail))),
            }?;
            
            indices.push(index);
            data = data_inner;
        };
        
        let (data, field) = String::deserialize(data)?;
        let (data, value) = Value::deserialize(data)?;
        
        Ok((data, Self { indices, field, value }))
    }
}