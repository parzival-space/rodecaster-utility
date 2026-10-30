use std::thread::sleep;
use hidapi::HidApi;
use log::{error, warn};
use vizia::prelude::*;

pub struct DeviceList {
    selected_device: Signal<Option<String>>,
    device_list: SyncSignal<Vec<String>>,
}

impl DeviceList {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        let selected_device: Signal<Option<String>> = Default::default();
        let device_list: SyncSignal<Vec<String>> = Default::default();

        let tokio = tokio::runtime::Handle::current();
        tokio.spawn_blocking(move || Self::run_hid_scan_thread(device_list));

        Self { selected_device, device_list }.build(cx, |cx| {

            List::new(cx, device_list, |cx, index, item| {
                Label::new(cx, item)
                    .hoverable(true);
            });
        })
    }

    fn run_hid_scan_thread(signal: SyncSignal<Vec<String>>) {
        let mut hid_api = match HidApi::new() {
            Ok(hid_api) => hid_api,
            Err(err) => {
                error!("Failed to initialize HidApi: {}", err);
                return;
            }
        };

        loop {
            if hid_api.refresh_devices().is_err() {
                warn!("Failed to refresh HID devices");
                sleep(Duration::from_millis(1000));
                continue;
            }

            signal.set(
                hid_api.device_list()
                    .map(|device| device.product_string().unwrap_or_default().to_string())
                    .collect()
            );

            sleep(Duration::from_millis(1000));
        }
    }
}

impl View for DeviceList {
    fn element(&self) -> Option<&'static str> {
        Some("device_list")
    }
}