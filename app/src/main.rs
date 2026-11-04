mod state;

use env_logger::{Builder, Env};
use tokio_util::sync::CancellationToken;
use vizia::{include_style, ApplicationError};
use vizia::prelude::{Application, Label, List, SignalMap, WindowModifiers};
use crate::state::hotplug::HotplugState;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    // some tasks need to be canceled when the application exits, so we allocate a cancellation token
    // in the future this shouldn't be necessary anymore as vizia will handle this for us using tasks
    let worker_token = CancellationToken::new();
    let worker_token_clone = worker_token.clone();

    let application_result = Application::new(move |cx| {
        cx.add_stylesheet(include_style!("resources/ui/main.css"))
            .expect("Failed to load main stylesheet");

        let connected_devices = Default::default();
        HotplugState { connected_devices }.build_and_run_worker(cx, worker_token_clone.clone());

        List::new(cx, connected_devices, |cx, _index, entry| {
            Label::new(cx, entry.map(|(location, device_type)| format!("{location} - {:?}", device_type)));
        });
    })
        .title("RODECaster Utility")
        .inner_size((800, 600))
        .run();

    // cancel running background tasks
    worker_token.cancel();

    application_result
}