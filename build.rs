fn main() {
    // cargo-bundle now handles .desktop file generation automatically
    // via the [package.metadata.bundle.linux] configuration in Cargo.toml
    
    // Rebuild if Cargo.toml changes (for bundle metadata updates)
    println!("cargo:rerun-if-changed=Cargo.toml");
}

