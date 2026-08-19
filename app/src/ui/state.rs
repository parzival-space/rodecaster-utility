use vizia::context::EventContext;
use vizia::events::Event;
use vizia::model::Model;
use vizia::prelude::SyncReadSignal;

#[derive(Debug, Default)]
pub struct AppState {
    device_list: SyncReadSignal<Vec<String>>, // todo: use proper device struct for this
    current_device: SyncReadSignal<Option<String>>, // todo: same as above, but also might need rw
    current_device_state: SyncReadSignal<Option<String>>, // todo: same as above
}

pub enum  AppEvent {
    SelectDevice(String),
}

impl Model for AppState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, meta| match app_event {
            AppEvent::SelectDevice(selected_device) => {
                // todo: send event to background worker
                meta.consume();
            }
        })
    }
}