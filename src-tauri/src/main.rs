#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    configure_linux_webkit_runtime();

    memory_pak_tauri::run();
}

#[cfg(all(target_os = "linux", not(debug_assertions)))]
fn configure_linux_webkit_runtime() {
    // WebKitGTK's accelerated compositor can abort() when an AppImage bundles
    // Ubuntu WebKitGTK but runs against a host GPU/Wayland stack from a rolling
    // distro such as Arch. These defaults keep release builds conservative, and
    // users can still override them by setting the environment themselves.
    set_env_default("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    set_env_default("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    if std::env::var_os("APPIMAGE").is_some() {
        set_env_default("GDK_BACKEND", "x11");
    }
}

#[cfg(not(all(target_os = "linux", not(debug_assertions))))]
fn configure_linux_webkit_runtime() {}

#[cfg(all(target_os = "linux", not(debug_assertions)))]
fn set_env_default(key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        std::env::set_var(key, value);
    }
}
