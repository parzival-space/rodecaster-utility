use nom::IResult;

mod device_report;
mod property_update_packet;

pub(crate) use device_report::DeviceReportPacket;
pub(crate) use property_update_packet::PropertyUpdatePacket;
use crate::error::UsbError;

pub trait RodeCasterPacket {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized;

    fn to_bytes(&self) -> Result<Vec<u8>, UsbError>;
}
