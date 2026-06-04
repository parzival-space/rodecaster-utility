use crate::protocol::packet::RodeCasterPacket;
use crate::protocol::packet::{DeviceReportPacket, PropertyUpdatePacket};
use crate::error::UsbError;

#[derive(Debug, Clone, PartialEq)]
pub enum PacketType {
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
pub fn parse_raw_packet(bytes: Vec<u8>) -> Result<PacketType, UsbError> {
    match bytes.first().copied() {
        Some(0x01) => {
            let result = PropertyUpdatePacket::from_bytes(&bytes)
                .map(|result| result.1)
                .or_else(|e| parse_nom_error(e))?;
            Ok(PacketType::PropertyUpdate(result))
        }
        Some(0x02) => {
            let result = DeviceReportPacket::from_bytes(&bytes)
                .map(|result| result.1)
                .or_else(|e| parse_nom_error(e))?;
            Ok(PacketType::DeviceReport(result))
        }
        _ => Ok(PacketType::Unknown(bytes)),
    }
}

fn parse_nom_error<T>(e: nom::Err<nom::error::Error<&[u8]>>) -> Result<T, UsbError> {
    match e {
        nom::Err::Incomplete(needed) => Err(UsbError::FrameParse(format!(
            "Failed to parse from packet data. Needed: {:?} bytes",
            needed))),
        nom::Err::Error(error) => {
            let next_20_bytes = error.input[..20].to_vec();
            let next_20_chars = String::from_utf8_lossy(&next_20_bytes);
            Err(UsbError::FrameParse(
                format!("Failed to parse from packet data. Error: {:?}. Next 20 chars: {:?} Next 20 bytes: {:?}. Code: {:?}",
                        error,
                        next_20_chars,
                        next_20_bytes,
                        error.code)
            ))
        }
        e => Err(UsbError::FrameParse(format!("Failed to parse from packet data: {}", e))),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::protocol::types::Value;
//
//     #[test]
//     fn test_parse_framed_message() {
//         let mut messages = vec![
//             &[0x02, 0x00, 0x00, 0x00, 0xFF],
//             &[0xAA, 0x00, 0x00, 0x00, 0x00],
//         ];
//
//         let packet = read_framed_message(|| -> Result<Vec<u8>> {
//             let current_message = messages[0];
//             messages.remove(0);
//             Ok(current_message.to_vec())
//         })
//         .unwrap();
//
//         // verify the correct packet type was determined
//         assert!(matches!(packet, RodeCasterPacketResult::Unknown(_)));
//
//         // verify the correct packet data was read
//         let RodeCasterPacketResult::Unknown(data) = packet else {
//             unreachable!()
//         };
//         assert_eq!(data, vec![0xFF, 0xAA]);
//     }
//
//     #[test]
//     fn test_write_framed_message() {
//         let packet = PropertyUpdatePacket {
//             indices: vec![1, 2],
//             name: "test".to_string(),
//             value: Value::U32(42),
//         };
//
//         let mut captured = Vec::new();
//         write_framed_message(Box::new(packet), 10, |data| {
//             captured.push(data.to_vec());
//             Ok(())
//         })
//         .unwrap();
//
//         assert_eq!(3, captured.len());
//         assert_eq!(
//             vec![
//                 vec![0x12, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x01, 0x01, 0x01],
//                 vec![0x02, 0x74, 0x65, 0x73, 0x74, 0x00, 0x01, 0x05, 0x2A, 0x00],
//                 vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
//             ],
//             captured
//         );
//     }
// }
