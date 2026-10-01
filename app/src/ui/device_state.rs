use vizia::context::EventContext;
use vizia::events::Event;
use vizia::prelude::{Model, Signal, SignalUpdate};

#[derive(Debug, Default, Clone)]
pub struct DeviceState {
    pub active_device: Signal<String>,
    pub devices: Signal<Vec<String>>
}

pub enum DeviceEvent {
    ActiveDeviceChanged(String),
    DeviceListChanged(Vec<String>),
}

impl Model for DeviceState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _meta| match app_event {
            DeviceEvent::ActiveDeviceChanged(device) => self.active_device.set(device.clone()),
            DeviceEvent::DeviceListChanged(devices) => self.devices.set(devices.clone()),
        })
    }
}