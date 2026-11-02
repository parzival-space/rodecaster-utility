use std::thread::sleep;
use std::time::Duration;
use hidapi::HidApi;
use log::{debug, error, warn};
use tokio_util::sync::CancellationToken;
use vizia::prelude::*;

pub(crate) struct DeviceList {
    selected_device: SyncSignal<Option<String>>,
    
    device_list: SyncSignal<Vec<String>>,
    device_task_token: CancellationToken,
}

impl DeviceList {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        let selected_device = SyncSignal::<Option<String>>::default();
        let device_list = SyncSignal::<Vec<String>>::default();
        
        // run hid scan thread in tokio
        let tokio = tokio::runtime::Handle::current();
        let device_task_token = CancellationToken::new();
        let device_task_token_clone = device_task_token.clone();
        tokio.spawn_blocking(move || Self::run_hid_scan_task(device_list, device_task_token_clone));
        
        Self { selected_device, device_list, device_task_token }.build(cx, |cx| {
            cx.add_stylesheet(include_style!("assets/ui/components/device_list.css"))
                .expect("Failed to load main view styles");

            List::new(cx, device_list, |cx, _index, item| {
                Label::new(cx, item)
                    .hoverable(true);
            });
        })
    }

    // todo: move this into the rodecaster-usb crate together with the refactoring for the hid scan logic
    fn run_hid_scan_task(signal: SyncSignal<Vec<String>>, cancel_token: CancellationToken) {
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
        Some("device-list")
    }
}

impl Drop for DeviceList {
    fn drop(&mut self) {
        self.device_task_token.cancel();
    }
}