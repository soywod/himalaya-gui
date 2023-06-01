#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use himalaya_gui::App;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Himalaya",
        options,
        Box::new(|cc| match App::new(cc) {
            Ok(app) => Box::new(app),
            Err(err) => {
                eprintln!("Error while running native app: {}", err);
                std::process::exit(1);
            }
        }),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(himalaya_gui::TemplateApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
