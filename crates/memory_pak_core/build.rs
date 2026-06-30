use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let database_dir = manifest_dir.join("..").join("..").join("database");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let destination = out_dir.join("catalog.bin");

    println!("cargo:rerun-if-changed=build.rs");
    let output = memory_pak_catalog::builder::compile_to_path(&database_dir, &destination)
        .unwrap_or_else(|error| panic!("catalog compilation failed: {error}"));
    for path in output.source_files {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
