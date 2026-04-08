mod c_string;
mod structured;
mod value;

pub use c_string::*;
use nom::IResult;
pub use structured::*;
pub use value::*;

pub trait StreamableType {
    fn parse_from_stream(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized;
    fn write_to_stream(&self, stream: &mut Vec<u8>) -> anyhow::Result<()>;
}
