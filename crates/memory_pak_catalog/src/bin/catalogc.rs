use std::path::PathBuf;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let database = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("database"));
    let destination = args.next().map(PathBuf::from);

    let result = if let Some(destination) = destination.as_deref() {
        memory_pak_catalog::builder::compile_to_path(&database, destination)
    } else {
        memory_pak_catalog::builder::compile_database(&database)
    };
    match result {
        Ok(output) => {
            println!(
                "catalog ok: {} consoles, {} games, {} collectibles, {} bytes",
                output.console_count,
                output.game_count,
                output.collectible_count,
                output.bytes.len()
            );
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
