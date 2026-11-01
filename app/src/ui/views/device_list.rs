use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::sleep;
use hidapi::HidApi;
use log::{debug, error, warn};
use tokio::select;
use tokio_util::sync::CancellationToken;
use vizia::prelude::*;

pub struct DeviceList {
    selected_device: Signal<Option<String>>,
    device_list: SyncSignal<Vec<String>>,

    hid_task_token: CancellationToken,
}

impl DeviceList {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        let selected_device: Signal<Option<String>> = Default::default();
        let device_list: SyncSignal<Vec<String>> = Default::default();

        let tokio = tokio::runtime::Handle::current();

        let hid_task_token = CancellationToken::new();
        let hid_task_token_clone = hid_task_token.clone();
        tokio.spawn_blocking(move || Self::run_hid_scan_thread(device_list, hid_task_token_clone));

        Self { selected_device, device_list, hid_task_token }.build(cx, |cx| {

            List::new(cx, device_list, |cx, index, item| {
                Label::new(cx, item)
                    .hoverable(true);
            });
        })
    }

    // todo: move this into the rodecaster-usb crate together with the refactoring for the hid scan logic
    fn run_hid_scan_thread(signal: SyncSignal<Vec<String>>, cancel_token: CancellationToken) {
        let mut hid_api = match HidApi::new() {
            Ok(hid_api) => hid_api,
            Err(err) => {
                error!("Failed to initialize HidApi: {}", err);
                return;
            }
        };

        while !cancel_token.is_cancelled() {
            match hid_api.refresh_devices() {
                Ok(_) => {
                    let device_list = hid_api.device_list();
                    signal.set(
                        device_list
                            .map(|device| device.product_string().unwrap_or_default().to_string())
                            .collect()
                    )
                },
                Err(err) => {
                    warn!("Failed to refresh HID devices: {}", err);
                }
            }
            sleep(Duration::from_millis(100));
        }

        debug!("HID scan thread exited")
    }
}

impl View for DeviceList {
    fn element(&self) -> Option<&'static str> {
        Some("device_list")
    }
}

impl Drop for DeviceList {
    fn drop(&mut self) {
        self.hid_task_token.cancel();
    }
}