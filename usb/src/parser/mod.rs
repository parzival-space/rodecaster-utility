mod single_field_update;
mod common;

use nom::{error_position, IResult};
use nom::error::ErrorKind;
use nom::number::streaming::le_u8;
use crate::parser::single_field_update::{parse_single_field_update, SingleFieldUpdate};

#[derive(Debug, Clone, PartialEq)]
pub enum RodeCasterPacket {
    SingleFieldUpdate(SingleFieldUpdate),
    StatusReport,
}

pub fn parse_rodecaster_packet(input: &[u8]) -> IResult<&[u8], RodeCasterPacket> {
    let (input, packet_kind) = le_u8(input)?;
    match packet_kind {
        0x01 => {
            let (input, data) = parse_single_field_update(input)?;
            Ok((input, RodeCasterPacket::SingleFieldUpdate(data)))
        }
        0x02 => {
            // todo: parse StatusReport packet
            Ok((input, RodeCasterPacket::StatusReport))
        }
        _ => Err(nom::Err::Error(error_position!(input, ErrorKind::Verify))),
    }
}