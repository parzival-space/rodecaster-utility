use std::process::exit;
use vizia::{include_style, Application, ApplicationError};
use vizia::icons::{ICON_LINE, ICON_MINIMIZE, ICON_UNDERLINE, ICON_WINDOW_MINIMIZE, ICON_X};
use vizia::model::Model;
use vizia::modifiers::ActionModifiers;
use vizia::prelude::WindowModifiers;
use vizia::views::{Button, HStack, Label, Sidebar, Svg, VStack};
use crate::ui::state::AppState;

pub fn run_app(state: AppState) -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("src/resources/style.css"))
            .expect("Failed to load stylesheet");
        state.build(cx);

        HStack::new(cx, |cx| {
            Sidebar::new(
                cx,
                |cx| {
                    // todo: replace with better icons
                    Button::new(cx, |cx| {
                        Svg::new(cx, ICON_X)
                    })
                        .on_press(|_| exit(0));
                    Button::new(cx, |cx| {
                        Svg::new(cx, ICON_MINIMIZE)
                    });
                },
                |cx| {

                },
                |cx| {
                    Label::new(cx, "v0.1.0");
                },
            );

            VStack::new(cx, |cx| {
                Label::new(cx, "RODECaster Utility");
                Label::new(cx, "No active device");
            });
        });

    })
        .title("RODECaster Utility")
        .inner_size((1200, 720))
        .min_inner_size(Some((1200, 720)))
        .decorations(false)
        .run()
}