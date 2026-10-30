use vizia::prelude::*;

pub struct MainView {
    active_device: SyncSignal<String>,
}

impl MainView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        let active_device = SyncSignal::<String>::default();
        active_device.set("Loading... Doing high computational work...".into());

        let tokio = tokio::runtime::Handle::current();
        tokio.spawn(async move {
            // sleep 1s
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            active_device.set("Hello World".to_string());
        });

        Self { active_device }.build(cx, move |cx| {
            // todo: add child elements
            Label::new(cx, active_device);
        })
    }
}

impl View for MainView {
    fn element(&self) -> Option<&'static str> {
        None
    }
}