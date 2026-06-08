use std::io::Write;
use byteorder::{WriteBytesExt, LE};
use nom::IResult;
use nom::number::streaming::{le_u16, le_u8, le_u32, le_f64};
use crate::protocol::Parseable;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    U32(u32),
    Bool(bool),
    F64(f64),
    String(String),
    Double(f64), // unknown type to be honest, can be stored in f64
    Array(Vec<Value>),
}

impl Parseable for Value {
    fn can_parse(data: &[u8]) -> bool {
        // requires at least 3 bytes
        // 1 byte for length_bytes
        // at least 1 byte for length
        // at least 1 byte for value_type
        data.len() >= 3
    }

    fn parse(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, length_bytes) = le_u8(data)?;
        let (data, length) = match length_bytes {
            1u8 => le_u8(data).map(|res| (res.0, res.1 as usize))?, // u8 for size
            2u8 => le_u16(data).map(|res| (res.0, res.1 as usize))?, // u16 for size
            _ => Err(nom::Err::Error(
                nom::error::Error::new(data, nom::error::ErrorKind::LengthValue)
            ))?, // invalid number of bytes
        };

        let (mut data, value_type) = le_u8(data)?;
        match value_type {
            1u8 => Ok(le_u32(data).map(|res| (res.0, Value::U32(res.1)))?), // u32
            2u8 => Ok((data, Value::Bool(true))), // bool true
            3u8 => Ok((data, Value::Bool(false))), // bool false
            4u8 => Ok(le_f64(data).map(|res| (res.0, Value::F64(res.1)))?), // f64
            5u8 => Ok(String::parse(data).map(|res| (res.0, Value::String(res.1)))?), // string
            6u8 => Ok(le_f64(data).map(|res| (res.0, Value::F64(res.1)))?), // double, f64 works for now

            8u8 => {
                let mut remaining_value_bytes = length - 1usize;
                let mut values = Vec::new();
                while remaining_value_bytes > 0 {
                    let (data_boxed, value) = Self::parse(data)?;
                    values.push(value);

                    // subtract the bytes we just parsed from the remaining bytes
                    remaining_value_bytes -= data.len().abs_diff(data_boxed.len());
                    data = data_boxed;
                }
                Ok((data, Value::Array(values)))
            }

            _ => Err(nom::Err::Error(
                nom::error::Error::new(data, nom::error::ErrorKind::Fail)
            ))?,
        }
    }

    fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Value::U32(value) => {
                writer.write_u8(1u8)?; // num length bytes (u8)
                writer.write_u8(5u8)?; // 1 byte (type) + 4 bytes (u32)
                writer.write_u8(1u8)?; // type byte for u32
                writer.write_u32::<LE>(*value)
            }
            Value::Bool(value) => {
                writer.write_u8(1u8)?; // num length bytes (u8)
                writer.write_u8(1u8)?; // 1 byte (type) + 0 bytes (bool value is encoded in type byte)
                match value {
                    true => writer.write_u8(2u8), // type byte for bool true
                    false => writer.write_u8(3u8), // type byte for bool false
                }
            }
            Value::F64(value) => {
                writer.write_u8(1u8)?; // num length bytes (u8)
                writer.write_u8(9u8)?; // 1 byte (type) + 8 bytes (f64)
                writer.write_u8(4u8)?; // type byte for f64
                writer.write_f64::<LE>(*value)
            }
            Value::String(value) => {
                // 1 byte/char for type, 1 byte/char for terminator
                if value.len() + 2 <= u8::MAX as usize {
                    writer.write_u8(1u8)?; // num length bytes (u8)
                    writer.write_u8((value.len() + 2) as u8)?;
                    writer.write_u8(5u8)?; // type byte for string
                    value.write_to(writer)
                } else if value.len() + 2 <= u16::MAX as usize {
                    writer.write_u8(2u8)?; // num length bytes (u16)
                    writer.write_u16::<LE>((value.len() + 2) as u16)?;
                    writer.write_u8(5u8)?; // type byte for string
                    value.write_to(writer)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "String value too long to serialize",
                    ))
                }
            }
            Value::Double(value) => {
                writer.write_u8(1u8)?; // num length bytes (u8)
                writer.write_u8(9u8)?; // 1 byte (type) + 8 bytes (f64)
                writer.write_u8(6u8)?; // type byte for double?
                writer.write_f64::<LE>(*value)
            }
            Value::Array(values) => {
                let mut serialized_values = Vec::new();
                for value in values {
                    // sadly, we have to cache the serialized values here to determine
                    // the size in bytes
                    value.write_to(&mut serialized_values)?;
                }

                // add 1 byte for type
                if serialized_values.len() + 1 <= u8::MAX as usize {
                    writer.write_u8(1u8)?; // num length bytes (u8)
                    writer.write_u8((serialized_values.len() + 1) as u8)?;
                    writer.write_u8(8u8)?; // type byte for array
                    writer.write_all(&serialized_values)
                } else if serialized_values.len() + 1 <= u16::MAX as usize {
                    writer.write_u8(2u8)?; // num length bytes (u16)
                    writer.write_u16::<LE>((serialized_values.len() + 1) as u16)?;
                    writer.write_u8(8u8)?; // type byte for array
                    writer.write_all(&serialized_values)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Array value too long to serialize",
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // captured values
    const CAPTURED_U32: [u8; 7] = [ 0x01, 0x05, 0x01, 0x03, 0x00, 0x00, 0x00 ]; // 3
    const CAPTURED_U32_PARSED: u32 = 3;

    const CAPTURED_BOOL_TRUE: [u8; 0x03] = [ 0x01, 0x01, 0x02 ]; // true
    const CAPTURED_BOOL_TRUE_PARSED: bool = true;

    const CAPTURED_BOOL_FALSE: [u8; 0x03] = [ 0x01, 0x01, 0x03 ]; // false
    const CAPTURED_BOOL_FALSE_PARSED: bool = false;

    const CAPTURED_F64: [u8; 0x0B] = [ 0x01, 0x09, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x70, 0xE7, 0x40 ]; // 48000
    const CAPTURED_F64_PARSED: f64 = 48000f64;

    const CAPTURED_STRING: [u8; 0x08] = [ 0x01, 0x06, 0x05, 0x2D, 0x31, 0x7C, 0x30, 0x00 ]; // "-1|0"
    const CAPTURED_STRING_PARSED: &str = "-1|0";

    const CAPTURED_ARRAY: [u8; 0x09] = [ 0x01, 0x07, 0x08, 0x01, 0x01, 0x02, 0x01, 0x01, 0x02 ]; // [true, true]
    const CAPTURED_ARRAY_PARSED: [bool; 2] = [true, true];

    #[test]
    fn test_parse_u32() {
        let result = Value::parse(&CAPTURED_U32);
        assert!(result.is_ok());

        let (rest, value) = result.unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(value, Value::U32(CAPTURED_U32_PARSED));
    }

    #[test]
    fn test_serialize_u32() {
        let mut stream = Vec::new();
        let result = Value::U32(CAPTURED_U32_PARSED).write_to(&mut stream);
        assert!(result.is_ok());
        assert_eq!(stream, CAPTURED_U32);
    }

    #[test]
    fn test_parse_bool_true() {
        let result = Value::parse(&CAPTURED_BOOL_TRUE);
        assert!(result.is_ok());

        let (rest, value) = result.unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(value, Value::Bool(CAPTURED_BOOL_TRUE_PARSED));
    }

    #[test]
    fn test_serialize_bool_true() {
        let mut stream = Vec::new();
        let result = Value::Bool(CAPTURED_BOOL_TRUE_PARSED).write_to(&mut stream);
        assert!(result.is_ok());
        assert_eq!(stream, CAPTURED_BOOL_TRUE);
    }

    #[test]
    fn test_parse_bool_false() {
        let result = Value::parse(&CAPTURED_BOOL_FALSE);
        assert!(result.is_ok());

        let (rest, value) = result.unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(value, Value::Bool(CAPTURED_BOOL_FALSE_PARSED));
    }

    #[test]
    fn test_serialize_bool_false() {
        let mut stream = Vec::new();
        let result = Value::Bool(CAPTURED_BOOL_FALSE_PARSED).write_to(&mut stream);
        assert!(result.is_ok());
        assert_eq!(stream, CAPTURED_BOOL_FALSE);
    }

    #[test]
    fn test_parse_f64() {
        let result = Value::parse(&CAPTURED_F64);
        assert!(result.is_ok());

        let (rest, value) = result.unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(value, Value::F64(CAPTURED_F64_PARSED));
    }

    #[test]
    fn test_serialize_f64() {
        let mut stream = Vec::new();
        let result = Value::F64(CAPTURED_F64_PARSED).write_to(&mut stream);
        assert!(result.is_ok());
        assert_eq!(stream, CAPTURED_F64);
    }

    #[test]
    fn test_parse_string() {
        let result = Value::parse(&CAPTURED_STRING);
        assert!(result.is_ok());

        let (rest, value) = result.unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(value, Value::String(CAPTURED_STRING_PARSED.to_string()));
    }

    #[test]
    fn test_serialize_string() {
        let mut stream = Vec::new();
        let result = Value::String(CAPTURED_STRING_PARSED.to_string()).write_to(&mut stream);
        assert!(result.is_ok());
        assert_eq!(stream, CAPTURED_STRING);
    }

    #[test]
    fn test_parse_array() {
        let result = Value::parse(&CAPTURED_ARRAY);
        assert!(result.is_ok());

        let (rest, value) = result.unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(value, Value::Array(CAPTURED_ARRAY_PARSED.to_vec().iter().map(|b| Value::Bool(*b)).collect()));
    }

    #[test]
    fn test_serialize_array() {
        let mut stream = Vec::new();
        let result = Value::Array(CAPTURED_ARRAY_PARSED.to_vec().iter().map(|b| Value::Bool(*b)).collect()).write_to(&mut stream);
        assert!(result.is_ok());
        assert_eq!(stream, CAPTURED_ARRAY);
    }
}