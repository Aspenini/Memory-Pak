#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK's DMA-BUF / EGL path often abort() on mismatch between bundled
    // WebKit (Ubuntu CI) and the host GPU stack (common on Arch + AppImage).
    // Must run before `tauri::Builder` initializes GTK/WebKit.
    #[cfg(all(target_os = "linux", not(debug_assertions)))]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    memory_pak_tauri::run();
}
