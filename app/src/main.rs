use std::env;

fn main() -> tauri::Result<()> {
    unsafe {
        // under wayland on linux gtkwebkit has a bug that causes the app to crash on startup,
        // setting the following flag disables the DMABUG renderer to work around this issue.
        #[cfg(target_os = "linux")]
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    tauri::Builder::default()
        .run(tauri::generate_context!())
}
