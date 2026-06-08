pub mod handle;
pub mod io;
pub mod state;
pub mod backend;

use crate::devices::{DeviceRegistryEntry, VID_RODE, DeviceType};

pub(crate) const PID_RODECASTER_PRO_II: &[u16] = &[0x0037, 0x0072, 0x0078, 0x0030, 0x0094, 0x0092];

pub(crate) const REGISTRY_ENTRY: DeviceRegistryEntry = DeviceRegistryEntry {
    vendor_id: VID_RODE,
    product_ids: PID_RODECASTER_PRO_II,
    device_type: DeviceType::RodeCasterProII,
};


#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;
    use hidapi::HidApi;
    use crate::devices::manager::DeviceIdentifier;
    use crate::devices::rodecaster_pro_ii::handle::RodeCasterProIIHandle;
    use super::*;

    /// This test is used as a dev tool to download the current device status
    /// of the RODECaster Pro II as a json file.
    #[test]
    #[ignore]
    pub fn rodecaster_pro_ii_status_json() {
        let hid_api = HidApi::new().expect("Failed to initialize HID API");
        let devices = hid_api.device_list().collect::<Vec<_>>();

        let rodecaster_pro_ii_devices = devices.iter()
            .filter(|device| device.vendor_id() == REGISTRY_ENTRY.vendor_id)
            .filter(|device| REGISTRY_ENTRY.product_ids.contains(&device.product_id()))
            .cloned()
            .collect::<Vec<_>>();
        assert!(!rodecaster_pro_ii_devices.is_empty());

        let handle = RodeCasterProIIHandle::open(DeviceIdentifier {
            device_type: DeviceType::RodeCasterProII,
            device_info: rodecaster_pro_ii_devices[0].clone()
        }).expect("Failed to open device");

        // wait for device to be ready
        let mut failed_counter = 0;
        while failed_counter < 10 {
            if handle.state_snapshot().root.is_some() {
                break;
            } else {
                failed_counter += 1;
                sleep(Duration::from_millis(100));
            }
        }

        // serialize status
        assert!(handle.state_snapshot().root.is_some());
        let root = handle.state_snapshot().root.unwrap();

        let json = serde_json::to_string_pretty(&root).expect("Failed to serialize status to JSON");
        println!("RodeCaster Pro II Status:\n{}", json);
    }
}