pub(crate) mod hid_transport;
pub(crate) mod types;
pub(crate) mod packet;

/// allows serializing the type into the byte array the rodecaster protocol uses
pub(crate) trait Serialize {
    fn can_serialize(&self) -> bool {
        Self::serialize(self).is_ok()
    }

    fn serialize(&self) -> std::io::Result<Vec<u8>>;
}

/// allows deserializing into types from the rodecaster protocol byte array
pub(crate) trait Deserialize {
    fn can_deserialize(data: &[u8]) -> bool where Self: Sized {
        // clone to prevent modifying the original byte stream
        let data_copy = data.to_owned();
        Self::deserialize(data_copy.as_slice()).is_ok()
    }

    fn deserialize(data: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized;
}