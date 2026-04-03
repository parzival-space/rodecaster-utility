use anyhow::Result;
use rodecaster_usb::parser::{parse_rodecaster_packet, single_field_update, RodeCasterPacket};
use rodecaster_usb::parser::common::FieldValue;
use rodecaster_usb::parser::single_field_update::SingleFieldUpdate;

#[test]
#[allow(non_snake_case)]
fn parser_parses_mixMute_packet() -> Result<()> {
    let data = include_bytes!("mock/parser/valid/mixMute.bin");

    let (remaining, result) = parse_rodecaster_packet(data)?;

    assert_eq!(remaining.len(), 0); // no remaining bytes to parse
    assert_eq!(result, RodeCasterPacket::SingleFieldUpdate(
        SingleFieldUpdate {
            indices: vec![317],
            field_name: "mixMute".to_string(),
            field_value: FieldValue::True,
        }
    ));

    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn parser_parses_mutePressed_packet() -> Result<()> {
    let data = include_bytes!("mock/parser/valid/mutePressed.bin");

    let (remaining, result) = parse_rodecaster_packet(data)?;

    assert_eq!(remaining.len(), 0); // no remaining bytes to parse
    assert_eq!(result, RodeCasterPacket::SingleFieldUpdate(
        SingleFieldUpdate {
            indices: vec![0, 48],
            field_name: "mutePressed".to_string(),
            field_value: FieldValue::False,
        }
    ));

    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn parser_parses_outputMonLevel_packet() -> Result<()> {
    let data = include_bytes!("mock/parser/valid/outputMonLevel.bin");

    let (remaining, result) = parse_rodecaster_packet(data)?;

    assert_eq!(remaining.len(), 0); // no remaining bytes to parse
    assert_eq!(result, RodeCasterPacket::SingleFieldUpdate(
        SingleFieldUpdate {
            indices: vec![11],
            field_name: "outputMonLevel".to_string(),
            field_value: FieldValue::F64(0.1499999612569809),
        }
    ));

    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn parser_parses_encoderSignal_packet() -> Result<()> {
    let data = include_bytes!("mock/parser/valid/encoderSignal.bin");

    let (remaining, result) = parse_rodecaster_packet(data)?;

    assert_eq!(remaining.len(), 0); // no remaining bytes to parse
    assert_eq!(result, RodeCasterPacket::SingleFieldUpdate(
        SingleFieldUpdate {
            indices: vec![4],
            field_name: "encoderSignal".to_string(),
            field_value: FieldValue::Struct(vec![
                FieldValue::U32(4294967291),
                FieldValue::False
            ]),
        }
    ));

    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn parser_parses_mixLevelWithAnchor_packet() -> Result<()> {
    let data = include_bytes!("mock/parser/valid/mixLevelWithAnchor.bin");

    let (remaining, result) = parse_rodecaster_packet(data)?;

    assert_eq!(remaining.len(), 0); // no remaining bytes to parse
    assert_eq!(result, RodeCasterPacket::SingleFieldUpdate(
        SingleFieldUpdate {
            indices: vec![265],
            field_name: "mixLevelWithAnchor".to_string(),
            field_value: FieldValue::String("0.338583|0.338583".to_string()),
        }
    ));

    Ok(())
}

#[test]
#[should_panic]
#[allow(non_snake_case)]
fn parser_parses_unsupportedType_packet() {
    let data = include_bytes!("mock/parser/invalid/unsupportedType.bin");

    // this test parses an unsupported type, 0x16, which should reach an unreachable code statement
    parse_rodecaster_packet(data).unwrap();
}