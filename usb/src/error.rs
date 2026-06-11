use thiserror::Error;

#[derive(Error, Debug)]
pub enum UsbError {
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),

    #[error("Frame parse error: {0}")]
    FrameParse(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol write error: {0}")]
    ProtocolWrite(String),

    #[error("Property update error: {0}")]
    PropertyError(String),
    
    #[error("Timeout")]
    Timeout,

    #[error("Protocol parse error: {0}")]
    NomParse(#[from] nom::Err<nom::error::Error<&'static[u8]>>),
}