use crate::AppWindow;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn bind(window: &AppWindow) {
    use slint::winit_030::WinitWindowAccessor;
    use slint::ComponentHandle;

    {
        let weak = window.as_weak();
        window.on_window_drag(move || {
            if let Some(window) = weak.upgrade() {
                let _ = window
                    .window()
                    .with_winit_window(|window| window.drag_window());
            }
        });
    }
    {
        let weak = window.as_weak();
        window.on_window_minimize(move || {
            if let Some(window) = weak.upgrade() {
                let _ = window
                    .window()
                    .with_winit_window(|window| window.set_minimized(true));
            }
        });
    }
    {
        let weak = window.as_weak();
        window.on_window_maximize(move || {
            if let Some(window) = weak.upgrade() {
                let _ = window
                    .window()
                    .with_winit_window(|window| window.set_maximized(!window.is_maximized()));
            }
        });
    }
    window.on_window_close(|| {
        let _ = slint::quit_event_loop();
    });
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn bind(window: &AppWindow) {
    window.on_window_close(|| {
        let _ = slint::quit_event_loop();
    });
}
