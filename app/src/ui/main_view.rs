use crate::ui::views::device_list::DeviceList;
use vizia::prelude::*;

pub struct MainView {
    active_device: SyncSignal<String>,
}

impl MainView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        let active_device = SyncSignal::<String>::default();

        Self { active_device }.build(cx, move |cx| {
            cx.add_stylesheet(include_style!("assets/ui/styles/main_view.css"))
                .expect("Failed to load main view styles");

            DeviceList::new(cx);
        })
    }
}

impl View for MainView {
    fn element(&self) -> Option<&'static str> {
        Some("main")
    }
}