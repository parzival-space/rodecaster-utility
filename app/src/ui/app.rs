use vizia::prelude::*;
use crate::ui::views::device_list::DeviceList;

pub struct MainView {
    active_device: SyncSignal<String>,
}

impl MainView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        let active_device = SyncSignal::<String>::default();

        Self { active_device }.build(cx, move |cx| {
            let _ =cx.add_stylesheet(include_style!("src/resources/ui/main.css"));

            DeviceList::new(cx);
        })
    }
}

impl View for MainView {
    fn element(&self) -> Option<&'static str> {
        Some("main")
    }
}