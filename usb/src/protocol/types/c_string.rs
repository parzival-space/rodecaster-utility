use byteorder::WriteBytesExt;
use nom::bytes::streaming::{tag, take_till};
use nom::IResult;
use std::io::Write;

/// Helper function that reads a C-style null terminated string.
/// The terminator byte is not included in the resulting string, but is consumed from the input.
pub(crate) fn parse_c_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, bytes) = take_till(|b| b == 0)(input)?;
    let (input, _) = tag([0u8].as_ref())(input)?;
    let string = String::from_utf8_lossy(bytes).to_string();
    Ok((input, string))
}

/// Helper function that writes a C-style null terminated string.
pub(crate) fn write_c_string(stream: &mut Vec<u8>, string: &String) -> anyhow::Result<()> {
    stream.write(string.as_bytes())?;
    stream.write_u8(0x00)?; // null terminator
    Ok(())
}
