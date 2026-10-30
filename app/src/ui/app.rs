use tokio::runtime;
use vizia::context::EventContext;
use vizia::events::Event;
use vizia::prelude::{Context, Handle, Label, Model, Signal, SignalUpdate, View};
use log::{debug, info};

pub enum MainViewEvent {
    Example(String)
}

#[derive(Clone)]
pub struct MainView {
    runtime: runtime::Handle,
    example: Signal<String>
}

impl MainView {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        let self_ = Self {
            runtime: runtime::Handle::current(),
            example: Signal::new("".into())
        };
        let self_ref = self_.clone();

        self_.build(cx, |cx| {

            self_ref.example.set("init".into());

            // set text using event on load
            let _ = cx.get_proxy().emit(MainViewEvent::Example("init".into()));

            // set text using event after 1s
            let mut proxy = cx.get_proxy();
            self_ref.runtime.spawn(async move {
                // sleep 1s
                info!("sleep 1s");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                let _ = proxy.emit(MainViewEvent::Example("spawn".into()));
                info!("event sent")
            });

            // display default text
            info!("init");
            Label::new(cx, self_ref.example);
        })
    }
}

impl View for MainView {
    fn element(&self) -> Option<&'static str> {
        Some("main")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.take(|event, _| match event {
            MainViewEvent::Example(value) => {
                self.example.set(value);
            }
        })
    }
}