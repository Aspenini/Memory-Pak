#![windows_subsystem = "windows"]

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn main() -> Result<(), eframe::Error> {
    memory_pak_core::run_memory_pak_native()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    memory_pak_core::run_memory_pak_web();
}

#[cfg(target_os = "android")]
fn main() {}
