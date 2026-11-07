use hidapi::DeviceInfo;
use std::ffi::CString;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct DeviceIdentifier {
    pub path: CString,
    pub serial: String,
    pub(crate) device_info: DeviceInfo,
}

impl From<DeviceInfo> for DeviceIdentifier {
    fn from(device_info: DeviceInfo) -> Self {
        DeviceIdentifier {
            path: device_info.path().to_owned(),
            serial: device_info.serial_number().unwrap_or_default().to_owned(),
            device_info,
        }
    }
}

impl Into<DeviceInfo> for DeviceIdentifier {
    fn into(self) -> DeviceInfo {
        self.device_info
    }
}

impl PartialEq for DeviceIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.serial == other.serial
    }
}

impl Display for DeviceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.path.to_string_lossy(), self.serial)
    }
}