use std::collections::{HashMap, HashSet};
use log::{debug, warn};
use tokio::{select, spawn};
use tokio::task::spawn_local;
use tokio_util::sync::CancellationToken;
use vizia::prelude::*;
use rodecaster_usb::{flume, hotplug};
use rodecaster_usb::hotplug::{watch_hid_hotplug_event, HidHotPlugEvent, DeviceType, HidHotPlugThreadManagement};

pub(crate) struct DeviceList {
    selected_device: Signal<Option<String>>,
    
    device_list: Signal<Vec<String>>,
    device_task_token: CancellationToken,
}

impl DeviceList {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        let selected_device: Signal<Option<String>> = Signal::default();
        let device_list: Signal<Vec<String>> = Signal::default();
        
        // run hid scan thread in tokio
        let device_task_token = CancellationToken::new();
        let device_task_token_clone = device_task_token.clone();
        
        Self { selected_device, device_list, device_task_token }.build(cx, |cx| {
            cx.add_stylesheet(include_style!("assets/ui/components/device_list.css"))
                .expect("Failed to load main view styles");

            // spawn hid scan task
            let proxy = cx.get_proxy();
            let tokio = tokio::runtime::Handle::current();
            tokio.spawn(Self::run_hid_scan_task(proxy, device_task_token_clone));

            List::new(cx, device_list, |cx, _index, item| {
                Label::new(cx, item)
                    .hoverable(true);
            });
        })
    }

    // todo: move this into the rodecaster-usb crate together with the refactoring for the hid scan logic
    async fn run_hid_scan_task(mut proxy: ContextProxy, cancel_token: CancellationToken) {
        let (hotplug_tx, hotplug_rx) = flume::unbounded();
        let (management_tx, management_rx) = flume::unbounded();

        debug!("Starting hid scan task");
        let hid_handle = spawn(watch_hid_hotplug_event(hotplug_tx, management_rx));

        loop {
            select! {
                Ok(event) = hotplug_rx.recv_async() => {
                    match event {
                        HidHotPlugEvent::DeviceAttached(location, device_type) => {
                            let _ = proxy.emit(HidHotPlugEvent::DeviceAttached(location, device_type));
                        }
                        HidHotPlugEvent::DeviceRemoved(location, device_type) => {
                            let _ = proxy.emit(HidHotPlugEvent::DeviceRemoved(location, device_type));
                        }
                        HidHotPlugEvent::ThreadStopped => {
                            warn!("HID scan task stopped. The UI will no longer receive device updates.");
                            break;
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        let _ = management_tx.send_async(HidHotPlugThreadManagement::Quit).await;
        let _ = hid_handle.await;
        debug!("HID scan task stopped");
    }
}

impl View for DeviceList {
    fn element(&self) -> Option<&'static str> {
        Some("device-list")
    }

    // todo: not sure if this is the right way to handle events or if we should use SyncSignals, also fix remove/add logic
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.take(|event, _| match event {
            HidHotPlugEvent::DeviceAttached(_location, device_type) => {
                debug!("Device attached: {:?}", device_type);
                self.device_list.write().push(format!("{:?}", device_type));
            }
            HidHotPlugEvent::DeviceRemoved(_location, device_type) => {
                self.device_list.write().retain(|device| device != &format!("{:?}", device_type));
            }
            _ => {}
        })
    }
}

impl Drop for DeviceList {
    fn drop(&mut self) {
        self.device_task_token.cancel();
    }
}