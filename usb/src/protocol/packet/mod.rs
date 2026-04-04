use anyhow::Result;
use nom::IResult;

mod property_update_packet;

pub(crate) use property_update_packet::PropertyUpdatePacket;

pub trait RodeCasterPacket {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> where Self: Sized;
    
    fn to_bytes(&self) -> Result<Vec<u8>>;
}