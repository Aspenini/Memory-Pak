mod controller;
mod persistence;
mod platform_window;
pub mod services;

pub use persistence::SaveStore;

use std::error::Error;
use std::rc::Rc;

use memory_pak_core::MemoryPakApp;
use slint::ComponentHandle;

slint::include_modules!();

pub fn run() -> Result<(), Box<dyn Error>> {
    let store = persistence::platform_store()?;
    let (app, load_error) = match store.load()? {
        Some(json) => match MemoryPakApp::from_save_json(&json) {
            Ok(app) => {
                store.save(&app.save_json()?)?;
                (app, None)
            }
            Err(error) => (
                MemoryPakApp::default(),
                Some(format!(
                    "Could not load the existing save; it was left unchanged: {error}"
                )),
            ),
        },
        None => (MemoryPakApp::default(), None),
    };
    run_app(app, Rc::from(store), load_error)
}

fn run_app(
    app: MemoryPakApp,
    store: Rc<dyn persistence::SaveStore>,
    load_error: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let window = AppWindow::new()?;
    window.set_native_desktop(cfg!(not(any(
        target_arch = "wasm32",
        target_os = "android",
        target_os = "ios"
    ))));
    window.set_compact(cfg!(any(target_os = "android", target_os = "ios")));
    controller::bind(&window, app, store);
    platform_window::bind(&window);
    if let Some(error) = load_error {
        window.set_error_message(error.into());
    }
    let size_timer = slint::Timer::default();
    let weak = window.as_weak();
    size_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(200),
        move || {
            if let Some(window) = weak.upgrade() {
                let slint_window = window.window();
                let size = slint_window.size();
                let scale = slint_window.scale_factor().max(1.0);
                window.set_compact(size.width as f32 / scale < 960.0);
                window.set_short_screen(size.height as f32 / scale < 600.0);
            }
        },
    );
    window.run()?;
    Ok(())
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: slint::android::AndroidApp) {
    persistence::set_android_app(android_app.clone());
    slint::android::init(android_app).expect("initialize Slint Android backend");
    run().expect("run Memory Pak");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let store: Rc<dyn persistence::SaveStore> = Rc::new(persistence::WebSaveStore);
        let (app, load_error) = match persistence::load_web_state().await {
            Ok(Some(json)) => match MemoryPakApp::from_save_json(&json) {
                Ok(app) => {
                    if let Ok(json) = app.save_json() {
                        let _ = store.save(&json);
                    }
                    (app, None)
                }
                Err(error) => (
                    MemoryPakApp::default(),
                    Some(format!(
                        "Could not load the existing save; it was left unchanged: {error}"
                    )),
                ),
            },
            Ok(None) => (MemoryPakApp::default(), None),
            Err(error) => {
                web_sys::console::warn_1(&error);
                (
                    MemoryPakApp::default(),
                    Some("IndexedDB save could not be read.".to_string()),
                )
            }
        };
        if let Err(error) = run_app(app, store, load_error) {
            web_sys::console::error_1(&error.to_string().into());
        }
    });
}
