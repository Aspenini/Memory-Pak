fn main() {
    if let Err(error) = memory_pak_app::run() {
        eprintln!("Memory Pak failed to start: {error}");
        std::process::exit(1);
    }
}
