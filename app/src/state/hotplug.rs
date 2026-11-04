use std::collections::HashMap;
use log::debug;
use tokio::select;
use tokio_util::sync::CancellationToken;
use vizia::context::EventContext;
use vizia::events::Event;
use vizia::prelude::{Context, ContextProxy, DataContext, Model, Signal, SignalWrite};
use rodecaster_usb::flume;
use rodecaster_usb::hotplug::{watch_hid_hotplug_event, DeviceType, HidHotPlugThreadManagement, HidHotPlugEvent};

#[derive(Default)]
pub struct HotplugState {
    pub connected_devices: Signal<Vec<(String, DeviceType)>>,
}

impl HotplugState {
    pub fn build_and_run_worker(self, cx: &mut Context, cancellation_token: CancellationToken) -> &Self {
        let proxy = cx.get_proxy();
        tokio::spawn(Self::run_hotplug_task(proxy, cancellation_token));

        self.build(cx)
    }

    async fn run_hotplug_task(mut cx: ContextProxy, cancellation_token: CancellationToken) {
        let (hotplug_tx, hotplug_rx) = flume::unbounded();
        let (management_tx, management_rx) = flume::unbounded();

        let hid_task_handle = tokio::spawn(watch_hid_hotplug_event(hotplug_tx, management_rx));

        loop {
            select! {
                Ok(event) = hotplug_rx.recv_async() => {
                    // redirect the hotplug event to the vizia context
                    let _ = cx.emit(event);
                }
                _ = cancellation_token.cancelled() => {
                    break;
                }
            }
        }

        let _ = management_tx.send_async(HidHotPlugThreadManagement::Quit).await;
        let _ = hid_task_handle.await;
        debug!("Hotplug thread quit")
    }
}

impl Model for HotplugState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.take(|event, _| match event {
            HidHotPlugEvent::DeviceAttached(location, device_type) => {
                debug!("Device attached: {:?} {:?}", location, device_type);
                self.connected_devices.write().push((location, device_type));
            }
            HidHotPlugEvent::DeviceRemoved(location, _device_type) => {
                debug!("Device removed: {:?} {:?}", location, _device_type);
                self.connected_devices.write().retain(|(key, _)| key != &location);
            }
            _ => {}
        })
    }
}