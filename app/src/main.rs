use slint::PlatformError;

slint::include_modules!();

fn main() -> Result<(), PlatformError> {
    let main_window = MainWindow::new()?;

    main_window.run()
}
