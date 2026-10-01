use std::thread;
use crossbeam_channel::{select, Receiver, Sender};
use hidapi::DeviceInfo;
use log::{info, warn};
use vizia::prelude::*;
use crate::devices::manager::DeviceEvent;

pub enum AppEvent {
    DeviceEvent(DeviceEvent)
}

#[derive(Debug, Default)]
pub struct AppState {
    pub devices: Signal<Vec<DeviceInfo>>,
    pub active_device: Signal<Option<DeviceInfo>>
}

impl Model for AppState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _meta| match app_event {
            AppEvent::DeviceEvent(event) => match event {
                DeviceEvent::DeviceAttached(device_info) => {
                    info!("Device attached: {:?}", device_info);
                }
                DeviceEvent::DeviceRemoved(device_info) => {
                    info!("Device removed: {:?}", device_info);
                }
            }
        })
    }
}

impl AppState {
    pub fn register_device_listener(mut cx: ContextProxy, device_event_receiver: Receiver<DeviceEvent>) {
        let device_event_receiver = device_event_receiver.clone();
        thread::spawn(move || {
            while let Ok(event) = device_event_receiver.recv() {
                if let Err(err) = cx.emit(event) {
                    warn!("Failed to emit device event: {}", err);
                }
            }
        });
    }
}