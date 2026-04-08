use crate::protocol::packet::RodeCasterPacket;
use crate::protocol::packet::{DeviceReportPacket, PropertyUpdatePacket};
use anyhow::Result;
use nom::bytes::streaming::take;
use nom::number::streaming::le_u32;
use std::cmp::min;

#[derive(Debug, Clone, PartialEq)]
pub enum RodeCasterPacketResult {
    Unknown(Vec<u8>),
    PropertyUpdate(PropertyUpdatePacket),
    DeviceReport(DeviceReportPacket),
}

// This file contain readers and writers for the framed message format that the RODECaster devices
// use. Frames can be larger than the supported HID Report size and are then split into multiple
// messages that can just be concatenated together.
//
// General Protocol Format:
// <REPORT ID> <u32 PACKET LENGTH> <PACKET DATA>
//
// The HID Report ID should never be passed to the parsing methods below as this might be a device
// specific ID that is not the same across RODECaster product.

/// Reader for framed RODECaster protocol messages, as they are usually send using HID reports.
pub fn read_framed_message<F>(mut get_next_frame: F) -> Result<RodeCasterPacketResult>
where
    F: FnMut() -> Result<Vec<u8>>,
{
    // first 4 bytes are the actual message length
    let mut current_frame = get_next_frame()?;
    let (first_read_remaining, packet_length) = le_u32::<&[u8], ()>(&current_frame.as_slice())?;

    // create message buffer and read remaining
    let mut packet = Vec::new();

    // read remaining packet data
    let mut packet_bytes_remaining = packet_length as usize;
    current_frame = first_read_remaining.to_vec();
    while packet_bytes_remaining > 0 {
        let bytes_read_target = min(packet_bytes_remaining, current_frame.len());
        let (_, bytes_read) =
            take::<usize, &[u8], ()>(bytes_read_target)(current_frame.as_slice())?;
        // we can ignore the remaining bytes as the report size is usually bigger than the last frame

        packet_bytes_remaining -= &bytes_read.len();
        packet.extend_from_slice(&bytes_read);

        if packet_bytes_remaining > 0 {
            current_frame = get_next_frame()?;
        }
    }

    match packet.first().copied() {
        Some(0x01) => {
            let result = PropertyUpdatePacket::from_bytes(&packet)
                .map(|result| result.1)
                .or_else(|e| parse_nom_error(e))?;
            Ok(RodeCasterPacketResult::PropertyUpdate(result))
        }
        Some(0x02) => {
            let result = DeviceReportPacket::from_bytes(&packet)
                .map(|result| result.1)
                .or_else(|e| parse_nom_error(e))?;
            Ok(RodeCasterPacketResult::DeviceReport(result))
        }
        _ => Ok(RodeCasterPacketResult::Unknown(packet.clone())),
    }
}

fn parse_nom_error<T>(e: nom::Err<nom::error::Error<&[u8]>>) -> Result<T> {
    match e {
        nom::Err::Incomplete(needed) => Err(anyhow::anyhow!(
            "Failed to parse from packet data. Needed: {:?} bytes",
            needed
        )),
        nom::Err::Error(error) => {
            let next_20_bytes = error.input[..20].to_vec();
            let next_20_chars = String::from_utf8_lossy(&next_20_bytes);
            Err(anyhow::anyhow!(
                "Failed to parse from packet data. Error: {:?}. Next 20 chars: {:?} Next 20 bytes: {:?}. Code: {:?}",
                error,
                next_20_chars,
                next_20_bytes,
                error.code
            ))
        }
        e => Err(anyhow::anyhow!("Failed to parse from packet data: {}", e)),
    }
}

/// Writer for framed RODECaster protocol messages, as they are usually send using HID reports.
pub fn write_framed_message<F>(
    packet: Box<dyn RodeCasterPacket>,
    frame_size: usize,
    mut write_next_frame: F,
) -> Result<()>
where
    F: FnMut(Vec<u8>) -> Result<()>,
{
    // create packet_bytes and prepend the length of packet_body as u32
    let packet_body_bytes = packet.to_bytes()?;
    let packet_bytes = Vec::new()
        .into_iter()
        .chain((packet_body_bytes.len() as u32).to_le_bytes().into_iter())
        .chain(packet_body_bytes.into_iter())
        .collect::<Vec<u8>>();

    for chunk in packet_bytes.chunks(frame_size) {
        let frame_data = {
            let mut frame_data = chunk.to_vec();
            if frame_data.len() < frame_size {
                // pad remaining frame with 0x00
                frame_data.append(&mut vec![0u8; frame_size - frame_data.len()]);
            }
            frame_data
        };

        write_next_frame(frame_data)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::Value;

    #[test]
    fn test_parse_framed_message() {
        let mut messages = vec![
            &[0x02, 0x00, 0x00, 0x00, 0xFF],
            &[0xAA, 0x00, 0x00, 0x00, 0x00],
        ];

        let packet = read_framed_message(|| -> Result<Vec<u8>> {
            let current_message = messages[0];
            messages.remove(0);
            Ok(current_message.to_vec())
        })
        .unwrap();

        // verify the correct packet type was determined
        assert!(matches!(packet, RodeCasterPacketResult::Unknown(_)));

        // verify the correct packet data was read
        let RodeCasterPacketResult::Unknown(data) = packet else {
            unreachable!()
        };
        assert_eq!(data, vec![0xFF, 0xAA]);
    }

    #[test]
    fn test_write_framed_message() {
        let packet = PropertyUpdatePacket {
            indices: vec![1, 2],
            name: "test".to_string(),
            value: Value::U32(42),
        };

        let mut captured = Vec::new();
        write_framed_message(Box::new(packet), 10, |data| {
            captured.push(data.to_vec());
            Ok(())
        })
        .unwrap();

        assert_eq!(3, captured.len());
        assert_eq!(
            vec![
                vec![0x12, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x01, 0x01, 0x01],
                vec![0x02, 0x74, 0x65, 0x73, 0x74, 0x00, 0x01, 0x05, 0x2A, 0x00],
                vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ],
            captured
        );
    }
}
