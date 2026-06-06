pub mod types;
pub mod packets;

use std::io::{Write, Result};
use nom::IResult;

pub(crate) trait Parseable {
    fn can_parse(data: &[u8]) -> bool;

    fn parse(data: &[u8]) -> IResult<&[u8], Self> where Self: Sized;

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()>;
}