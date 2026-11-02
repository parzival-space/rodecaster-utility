use vizia::context::Context;
use vizia::prelude::{Handle, View, include_style};
use crate::components::device_list::DeviceList;

pub struct MainView;

impl MainView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self { }.build(cx, move |cx| {
            cx.add_stylesheet(include_style!("assets/ui/views/main_view.css"))
                .expect("Failed to load main view styles");
            
            DeviceList::new(cx);
        })
    }
}

impl View for MainView {
    fn element(&self) -> Option<&'static str> {
        Some("main-view")
    }
}