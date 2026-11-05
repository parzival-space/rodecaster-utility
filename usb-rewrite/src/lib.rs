pub mod hotplug;
pub mod device;
pub mod common;

/// Generic error type for the entire library
#[derive(Debug, thiserror::Error)]
pub enum RodeCasterError {
    #[error("HidApi error: {0:?}")]
    HidApiError(#[from] hidapi::HidError),
}