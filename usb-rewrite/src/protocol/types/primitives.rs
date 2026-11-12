use std::io::Write;
use byteorder::{WriteBytesExt, LE};
use nom::{IResult};
use nom::bytes::streaming::*;
use crate::protocol::{Deserialize, Serialize};

impl Serialize for String {
    fn serialize(&self) -> std::io::Result<Vec<u8>>
    {
        let mut stream = vec![];
        stream.write_all(self.as_bytes())?;
        stream.write_u8(0x00)?; // null terminator
        Ok(stream)
    }
}

impl Deserialize for String {
    fn deserialize(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, string_bytes) = take_till(|b| b == 0x00)(data)?;
        let (data, _tag) = tag([0u8].as_ref())(data)?;
        let string = String::from_utf8_lossy(string_bytes).to_string();
        Ok((data, string))
    }
}