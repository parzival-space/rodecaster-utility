use log::{LevelFilter, SetLoggerError};
use simplelog::{Config, TermLogger};
use vizia::ApplicationError;
use vizia::context::EventContext;
use vizia::events::Event;
use vizia::prelude::{ActionModifiers, Alignment, Application, Color, DataContext, EmitContext, Gradient, HStack, HorizontalPositionKeyword, Label, LayoutModifiers, Length, LineDirection, LinearGradient, Model, Percentage, Pixels, Signal, SignalGet, SignalUpdate, StyleModifiers, VStack, WindowModifiers};
use vizia::style::VerticalPositionKeyword::Bottom;
use vizia::vg::shaders::linear_gradient;
use vizia::views::Button;
use crate::ui::app::App;

mod ui;

#[derive(Debug)]
enum AppError {
    ApplicationError(ApplicationError),
    SetLoggerError(SetLoggerError)
}

#[derive(Debug, Default)]
struct AppData {
    count: Signal<i32>
}

enum AppEvent {
    Increment,
    Decrement
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::Increment => self.count.update(|count| *count += 1),
            AppEvent::Decrement => self.count.update(|count| *count -= 1)
        })
    }
}

fn main() -> Result<(), AppError> {
    TermLogger::init(LevelFilter::Debug, Default::default(), Default::default(), Default::default())
        .map_err(|err| AppError::SetLoggerError(err))?;

    Application::new(|ctx| {
        AppData::default().build(ctx);

        HStack::new(ctx, |ctx| {
            Button::new(ctx, |ctx| Label::new(ctx, "Increment"))
                .on_press(|ctx| ctx.emit(AppEvent::Increment));

            Label::new(ctx, ctx.data::<AppData>().count);

            Button::new(ctx, |ctx| Label::new(ctx, "Decrement"))
                .on_press(|ctx| ctx.emit(AppEvent::Decrement));
        })
            .alignment(Alignment::Center)
            .horizontal_gap(Pixels(50.0));

    })
        .title("Hello World")
        .inner_size((400, 100))
        .run()
        .map_err(|err| AppError::ApplicationError(err))
}
