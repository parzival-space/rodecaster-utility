use nom::bytes::streaming::{tag, take_till};
use nom::error::Error;
use nom::IResult;
use nom::number::streaming::{le_f64, le_u16, le_u32, le_u8};

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Unknown1,               // 0x00
    U32(u32),               // 0x01
    True,                   // 0x02
    False,                  // 0x03
    F64(f64),               // 0x04
    String(String),         // 0x05
    Double(f64),            // 0x06
    Unknown2,               // 0x07
    Struct(Vec<FieldValue>), // 0x08
}

pub fn parse_null_terminated_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, bytes) = take_till(|b| b == 0x00)(input)?;
    let (input, _) = tag([0x00].as_ref())(input)?;

    let string = String::from_utf8_lossy(bytes).to_string();
    Ok((input, string))
}

pub fn parse_field_value(input: &[u8]) -> IResult<&[u8], FieldValue> {
    let (input, data_length_type) = le_u8(input)?; // reserved byte
    let (input, data_length) = match data_length_type {
        0x01 => le_u8(input).map(|res| (res.0, res.1 as usize))?,
        0x02 => le_u16(input).map(|res| (res.0, res.1 as usize))?,
        _ => Err(nom::Err::Error(Error::new(input, nom::error::ErrorKind::Verify)))?
    };

    let (input, data_type) = le_u8(input)?;
    match data_type {
        // u32
        0x01 => {
            // data_length includes the type byte, so actual data_length - 1
            let data_bytes = (data_length - 1) as usize;
            if data_bytes != 4 {
                // this is not an u32, something is wrong
                return Err(nom::Err::Error(Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )))
            }

            let (input, value) = le_u32(input)?;
            Ok((input, FieldValue::U32(value)))
        },

        // boolean
        0x02 => Ok((input, FieldValue::True)),
        0x03 => Ok((input, FieldValue::False)),

        // f64
        0x04 => {
            let data_bytes = (data_length - 1) as usize;
            if data_bytes != 8 {
                // this is not a double, something is wrong
                return Err(nom::Err::Error(Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )))
            }

            let (input, value) = le_f64(input)?;
            Ok((input, FieldValue::F64(value)))
        }

        // string
        0x05 => {
            let (input, value) = parse_null_terminated_string(input)?;
            Ok((input, FieldValue::String(value)))
        }

        // double
        0x06 => {
            let data_bytes = (data_length - 1) as usize;
            if data_bytes != 8 {
                return Err(nom::Err::Error(Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )))
            }

            let (input, value) = le_f64(input)?;
            Ok((input, FieldValue::Double(value)))
        }

        // struct
        0x08 => { // struct, contains multiple other values
            let mut remaining_data_bytes = (data_length - 1) as usize;

            let mut values = vec![];
            let mut last_input = input; // store the last input to pass to the next parser
            while remaining_data_bytes > 0 {
                let (input, value) = parse_field_value(last_input)?;
                values.push(value);

                // re-calculate remaining bytes and update last input reference
                remaining_data_bytes -= last_input.len().abs_diff(input.len());
                last_input = input;
            }
            Ok((last_input, FieldValue::Struct(values)))
        }

        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null_terminated_string() {
        let data = vec![b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd', 0x00];

        let (remaining, parsed) = parse_null_terminated_string(&data[..]).unwrap();
        assert_eq!(remaining, &[]);
        assert_eq!(parsed, "Hello World".to_string());
    }

    #[test]
    fn test_parse_field_value() {
        let data = vec![
            0x01, // reserved
            0x05, // 5 data bytes
            0x01, // u32
            0x10, 0x00, 0x00, 0x00 // 16 u32 (LE)
        ];

        let (remaining, parsed) = parse_field_value(&data).unwrap();
        assert_eq!(remaining, &[]);
        assert_eq!(parsed, FieldValue::U32(16));
    }
}