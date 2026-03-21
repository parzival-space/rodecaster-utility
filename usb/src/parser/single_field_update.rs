use nom::{IResult, Parser};
use nom::combinator::verify;
use nom::bytes::streaming::tag;
use nom::number::streaming::{le_u16, le_u8};
use crate::parser::common::{parse_field_value, parse_null_terminated_string, FieldValue};

/// Represents a single field update message from the RodeCaster device, containing the header information and the updated value.
#[derive(Debug, Clone, PartialEq)]
pub struct SingleFieldUpdate {
    pub header_field_type: u16,
    pub header_field_value: u8,
    pub field_name: String,
    pub field_value: FieldValue,
}

// todo: replace with proper enums
fn parse_header_field_type(input: &[u8]) -> IResult<&[u8], (u8, u16)> {
    let (input, type_length) = verify(le_u8, |&len| len == 0x01 || len == 0x02)
        .parse(input)?;

    let (input, field_type) = match type_length {
        0x01 => {
            let (input, byte) = le_u8(input)?;
            (input, byte as u16)
        }
        0x02 => le_u16(input)?,
        _ => unreachable!(), // This case is handled by the verify combinator
    };

    Ok((input, (type_length, field_type)))
}

pub fn parse_single_field_update(input: &[u8]) -> IResult<&[u8], SingleFieldUpdate> {
    let (input, _reserved) = tag([0x01].as_ref())(input)?; // reserved byte

    let (input, (_type_length, header_field_type)) = parse_header_field_type(input)?;
    let (input, header_field_value) = le_u8(input)?;
    let (input, field_name) = parse_null_terminated_string(input)?;
    let (input, field_value) = parse_field_value(input)?;

    Ok((input, SingleFieldUpdate {
        header_field_type,
        header_field_value,
        field_name,
        field_value,
    }))
}