mod value;
mod structured;
mod c_string;

use nom::IResult;
pub use value::*;
pub use structured::*;
pub use c_string::*;

pub trait StreamableType {
    fn parse_from_stream(input: &[u8]) -> IResult<&[u8], Self> where Self: Sized;
    fn write_to_stream(&self, stream: &mut Vec<u8>) -> anyhow::Result<()>;
}