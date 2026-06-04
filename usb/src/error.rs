use thiserror::Error;

#[derive(Error, Debug)]
pub enum UsbError {
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),

    #[error("Frame parse error: {0}")]
    FrameParse(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}