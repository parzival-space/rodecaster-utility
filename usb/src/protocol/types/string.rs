use std::io::{Result, Write};
use byteorder::WriteBytesExt;
use nom::bytes::streaming::{take_till, tag};
use nom::IResult;
use crate::protocol::Parseable;

/// Helper implementation of Parseable for the String type.
/// This basically parses the String as a null terminated ASCII string.
impl Parseable for String {
    fn can_parse(data: &[u8]) -> bool {
        // must contain at least one byte
        data.len() > 0
    }

    fn parse(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, bytes) = take_till(|b| b == 0x00)(data)?;
        let (data, _) = tag([0x00u8].as_ref())(data)?;
        let string = String::from_utf8_lossy(bytes).to_string();
        Ok((data, string))
    }

    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(self.as_bytes())?;
        writer.write_u8(0x00) // null terminator
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;
    use super::*;

    const VALID_BYTES: [u8; 6] = [72u8, 101u8, 108u8, 108u8, 111u8, 0u8]; // "Hello\0"
    const INVALID_BYTES: [u8; 0] = [];

    #[test]
    fn test_can_parse_empty_string() {
        let result = String::can_parse(&INVALID_BYTES);
        assert_eq!(result, false);
    }

    #[test]
    fn test_can_parse_string() {
        let result = String::can_parse(&VALID_BYTES);
        assert_eq!(result, true);
    }

    #[test]
    fn test_parse_string() {
        let result = String::parse(&VALID_BYTES);
        assert!(result.is_ok());

        let (remaining, parsed_string) = result.unwrap();
        assert_eq!(parsed_string, "Hello");
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_parse_empty_string_err() {
        let data = [];

        let result = String::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let input = "Hello".to_string();
        let expected = [72u8, 101u8, 108u8, 108u8, 111u8, 0u8];

        let mut stream = Vec::new();
        let result = input.write_to(&mut stream);
        assert!(result.is_ok());

        assert_eq!(stream, expected);
    }
}