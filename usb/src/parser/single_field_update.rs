use log::{error, info};
use nom::{IResult, Parser};
use nom::combinator::verify;
use nom::bytes::streaming::{tag, tag_no_case};
use nom::number::streaming::{le_u16, le_u8};
use crate::parser::common::{parse_field_value, parse_null_terminated_string, FieldValue};

/// Represents a single field update message from the RodeCaster device, containing the header information and the updated value.
#[derive(Debug, Clone, PartialEq,)]
pub struct SingleFieldUpdate {
    pub indices: Vec<usize>,
    pub field_name: String,
    pub field_value: FieldValue,
}

pub fn parse_single_field_update(input: &[u8]) -> IResult<&[u8], SingleFieldUpdate> {
    // first byte seems to be always 0x01. it's very likely a length indicator for the next
    // indices_count byte. until we have found an actual packet using something different here
    // we will just assume that the first byte is always 0x01.
    let (input, _indices_count_length) = tag_no_case([0x01].as_ref())(input)?;

    let (input, indices_count) = le_u8(input)?;
    let (input, indices) = parse_indices(input, indices_count as usize)?;

    let (input, field_name) = parse_null_terminated_string(input)?;
    let (input, field_value) = parse_field_value(input)?;

    Ok((input, SingleFieldUpdate {
        indices,
        field_name,
        field_value,
    }))
}

fn parse_indices(input: &[u8], indices_count: usize) -> IResult<&[u8], Vec<usize>> {
    let mut indices = Vec::with_capacity(indices_count as usize);

    let mut last_input = input;
    for _ in 0..indices_count {
        let (input, index_length) = le_u8(last_input)?;
        let (input, index) = match index_length {
            0x00 => (input, 0), // no index value follows, just zero
            0x01 => le_u8(input).map(|res| (res.0, res.1 as usize))?,
            0x02 => le_u16(input).map(|res| (res.0, res.1 as usize))?,
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?
        };
        last_input = input;
        indices.push(index);
    }

    Ok((last_input, indices))
}