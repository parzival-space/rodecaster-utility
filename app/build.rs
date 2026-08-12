use std::collections::HashMap;
use std::path::PathBuf;
use slint_build::CompilerConfiguration;

fn main() {
    let library = HashMap::from([
        ("lucide".into(), PathBuf::from(lucide_slint::lib()))
    ]);

    let config = CompilerConfiguration::default()
        .with_library_paths(library);
    slint_build::compile_with_config("./ui/app-window.slint", config).unwrap();
}