use eframe::egui::ViewportBuilder;
use eframe::{NativeOptions, SurfaceConfig, WgpuConfiguration};
use eframe::wgpu::PresentMode;
use log::{LevelFilter, SetLoggerError};
use simplelog::{Config, TermLogger};
use crate::ui::app::App;

mod ui;

#[derive(Debug)]
enum RodeCasterUtilityError {
    EframeError(eframe::Error),
    SetLoggerError(SetLoggerError)
}

fn main() -> Result<(), RodeCasterUtilityError> {
    TermLogger::init(LevelFilter::Debug, Default::default(), Default::default(), Default::default())
        .map_err(|err| RodeCasterUtilityError::SetLoggerError(err))?;

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_decorations(true)
            .with_min_inner_size([1200.0, 720.0])
            .with_inner_size([1200.0, 720.0]),
        wgpu_options: WgpuConfiguration {
            surface: SurfaceConfig {
                present_mode: PresentMode::AutoVsync,
                desired_maximum_frame_latency: None,
            },
            ..Default::default()
        },

        ..Default::default()
    };
    eframe::run_native(
        "RODECaster Utility",
        native_options,
        Box::new(|_cc| Ok(Box::<App>::default()))
    )
        .map_err(|err| RodeCasterUtilityError::EframeError(err))
}
