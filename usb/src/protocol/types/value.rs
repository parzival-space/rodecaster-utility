use anyhow::bail;
use byteorder::{LittleEndian, WriteBytesExt};
use log::warn;
use nom::bytes::streaming::{take};
use nom::error::{Error, ErrorKind};
use nom::IResult;
use nom::number::streaming::{le_f64, le_u16, le_u32, le_u8};
use crate::protocol::types::{StreamableType, parse_c_string, write_c_string};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    U32(u32),
    Bool(bool),
    F64(f64),
    String(String),
    Double(f64),
    Combined(Vec<Value>),
    Unknown(Vec<u8>),
}

impl StreamableType for Value {
    fn parse_from_stream(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, data_length_type) = le_u8(input)?;
        let (input, data_length) = match data_length_type {
            0x01 => le_u8(input).map(|res| (res.0, res.1 as usize))?,
            0x02 => le_u16(input).map(|res| (res.0, res.1 as usize))?,
            _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Verify)))?
        };

        let (input, data_type) = le_u8(input)?;
        match data_type {
            // u32
            0x01 => {
                assert_eq!(data_length - 1, 4);
                let (input, value) = le_u32(input)?;
                Ok((input, Value::U32(value)))
            },
            // boolean
            0x02 => {Ok((input, Value::Bool(true)))},
            0x03 => {Ok((input, Value::Bool(false)))},
            // f64
            0x04 => {
                assert_eq!(data_length - 1, 8);
                let (input, value) = le_f64(input)?;
                Ok((input, Value::F64(value)))
            },
            // string
            0x05 => {
                let (input, value) = parse_c_string(input)?;
                Ok((input, Value::String(value)))
            },
            // double, f64 again but tagged differently
            0x06 => {
                assert_eq!(data_length - 1, 8);
                let (input, value) = le_f64(input)?;
                Ok((input, Value::Double(value)))
            },
            // Combined
            0x08 => {
                let mut remaining_data_bytes = data_length - 1;
                let mut values = vec![];
                let mut last_input = input;
                while remaining_data_bytes > 0 {
                    let (input, value) = Self::parse_from_stream(last_input)?;
                    values.push(value);

                    // re-calculate remaining bytes and update last input reference
                    remaining_data_bytes -= last_input.len().abs_diff(input.len());
                    last_input = input;
                }

                Ok((last_input, Value::Combined(values)))
            },

            _ => {
                warn!("Received unknown data type: {data_type}, with length: {data_length}. This is most likely due to a protocol change.");
                let (input, value) = take(data_length)(input)?;
                Ok((input, Value::Unknown(value.to_vec())))
            }
        }
    }

    fn write_to_stream(&self, stream: &mut Vec<u8>) -> anyhow::Result<()> {
        match self {
            // 0x01
            Value::U32(value) => {
                stream.write_u8(0x01)?; // data length type: u8
                stream.write_u8(0x05)?; // data length: 1 byte for type + 4 bytes for u32
                stream.write_u32::<LittleEndian>(*value)?;
            }
            // 0x02, 0x03
            Value::Bool(value) => {
                stream.write_u8(0x01)?; // data length type: u8
                stream.write_u8(0x01)?; // data length: 1 byte for type
                if *value {
                    stream.write_u8(0x02)?; // true
                } else {
                    stream.write_u8(0x03)?; // false
                }
            }
            // 0x04
            Value::F64(value) => {
                stream.write_u8(0x01)?; // data length type: u8
                stream.write_u8(0x09)?; // data length: 1 byte for type + 8 bytes for f64
                stream.write_f64::<LittleEndian>(*value)?;
            }
            // 0x05
            Value::String(value) => {
                if value.len() + 2 <= u8::MAX as usize { // +1 type, +1 null terminator
                    stream.write_u8(0x01)?; // data length type: u8
                    stream.write_u8((value.len() + 2) as u8)?; // data length: 1 byte for type + string_length
                    write_c_string(stream, value)?;
                } else if value.len() + 2 <= u16::MAX as usize {
                    stream.write_u8(0x02)?; // data length type: u8
                    stream.write_u16::<LittleEndian>((value.len() + 2) as u16)?; // data length: 1 byte for type + string_length
                    write_c_string(stream, value)?;
                } else {
                    bail!("String value is too long to be written in the current protocol implementation.");
                }
            }
            // 0x06
            Value::Double(value) => {
                stream.write_u8(0x01)?; // data length type: u8
                stream.write_u8(0x09)?; // data length: 1 byte for type + 8 bytes for f64
                stream.write_f64::<LittleEndian>(*value)?;
            }
            // 0x08
            Value::Combined(value) => {
                for value in value {
                    value.write_to_stream(stream)?;
                }
            }
            Value::Unknown(value) => {
                bail!("Cannot write unknown value type with length {}. Don't try to execute unsupported operations!", value.len());
            }
        };
        Ok(())
    }
}