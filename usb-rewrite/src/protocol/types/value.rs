use std::io::Write;
use byteorder::{WriteBytesExt, LE};
use nom::{error_position, IResult};
use nom::number::streaming::*;
use nom::error::ErrorKind;
use crate::protocol::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    U32(u32),
    Bool(bool),
    F64(f64),
    String(String),
    Double(f64), // todo: update this to i64, have to verify this tho
    List(Vec<Value>),
}

impl Serialize for Value {
    fn serialize(&self) -> std::io::Result<Vec<u8>>
    {
        let mut stream = vec![];
        match self {
            Value::U32(value) => {
                stream.write_u8(1u8)?; // num length bytes (u8 = 1 byte)
                stream.write_u8(5u8)?; // payload length (1 type + 4 value (u32))
                stream.write_u8(1u8)?; // protocol ID of u32
                stream.write_u32::<LE>(*value)?;
            }
            Value::Bool(value) => {
                stream.write_u8(1u8)?; // num length bytes (u8 = 1 byte)
                stream.write_u8(1u8)?; // payload length (1 type + 0 value), bools have their own type
                match value {
                    true => stream.write_u8(2u8)?,  // protocol ID for true
                    false => stream.write_u8(3u8)?, // protocol ID for false
                }
            }
            Value::F64(value) => {
                stream.write_u8(1u8)?; // num length bytes (u8 = 1 byte)
                stream.write_u8(9u8)?; // payload length (1 type + 8 value (f64))
                stream.write_u8(4u8)?; // protocol ID of f64
                stream.write_f64::<LE>(*value)?;
            }
            Value::String(value) => {
                let value_bytes = value.serialize()?;

                // add 1 byte for type ID
                match value_bytes.len() + 1 {
                    length if length <= u8::MAX as usize => {
                        stream.write_u8(1u8)?; // num length bytes (u8 = 1 byte)
                        stream.write_u8(length as u8)?;
                        stream.write_u8(5u8)?; // protocol ID of String
                        stream.write_all(&value_bytes)?;
                    }
                    length if length <= u16::MAX as usize => {
                        stream.write_u8(2u8)?; // num length bytes (u16 = 2 bytes)
                        stream.write_u16::<LE>(length as u16)?;
                        stream.write_u8(5u8)?; // protocol ID of String
                        stream.write_all(&value_bytes)?;
                    }
                    _ => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "String value too long to serialize",
                        ));
                    }
                }
            }
            Value::Double(value) => {
                stream.write_u8(1u8)?; // num length bytes (u8 = 1 byte)
                stream.write_u8(9u8)?; // payload length (1 type + 8 value (f64))
                stream.write_u8(6u8)?; // protocol ID of double?
                stream.write_f64::<LE>(*value)?;
            }
            Value::List(values) => {
                let mut values_bytes = vec![];
                for value in values {
                    // sadly, we have to cache the serialized values here to determine the size in bytes
                    value.serialize()?.write_all(&mut values_bytes)?;
                }

                // add 1 byte for type ID
                match values_bytes.len() + 1 {
                    length if length <= u8::MAX as usize => {
                        stream.write_u8(1u8)?; // num length bytes (u8 = 1 byte)
                        stream.write_u8(length as u8)?;
                        stream.write_u8(8u8)?; // protocol ID of List
                        stream.write_all(&values_bytes)?;
                    }
                    length if length <= u16::MAX as usize => {
                        stream.write_u8(2u8)?; // num length bytes (u16 = 2 bytes)
                        stream.write_u8(length as u8)?;
                        stream.write_u8(8u8)?; // protocol ID of List
                        stream.write_all(&values_bytes)?;
                    }
                    _ => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "List value too long to serialize",
                        ));
                    }
                }
            }
        }
        Ok(stream.to_vec())
    }
}

impl Deserialize for Value {
    fn deserialize(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, length_bytes) = le_u8(data)?;
        let (data, length) = match length_bytes {
            1u8 => le_u8(data).map(|(data, length)| (data, length as usize)),
            2u8 => le_u16(data).map(|(data, length)| (data, length as usize)),
            _ => Err(nom::Err::Error(error_position!(data, ErrorKind::LengthValue)))
        }?;

        let (mut data, value_type) = le_u8(data)?;
        match value_type {
            1u8 => Ok(le_u32(data).map(|(data, value)| (data, Value::U32(value)))?),
            2u8 => Ok((data, Value::Bool(true))),
            3u8 => Ok((data, Value::Bool(false))),
            4u8 => Ok(le_f64(data).map(|(data, value)| (data, Value::F64(value)))?),
            5u8 => Ok(String::deserialize(data).map(|(data, value)| (data, Value::String(value)))?),
            6u8 => Ok(le_f64(data).map(|(data, value)| (data, Value::Double(value)))?),

            8u8 => {
                let mut remaining_value_bytes = length - 1usize; // subtract 1 value_type byte
                let mut values = vec![];

                while remaining_value_bytes > 0 {
                    let (data_inner, value) = Self::deserialize(data)?;
                    values.push(value);

                    remaining_value_bytes -= data.len().abs_diff(data_inner.len());
                    data = data_inner;
                }
                Ok((data, Value::List(values)))
            }

            _ => Err(nom::Err::Error(error_position!(data, ErrorKind::Fail)))
        }
    }
}