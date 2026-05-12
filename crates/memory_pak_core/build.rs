use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[path = "src/compiled.rs"]
mod compiled;

use compiled::{
    CompiledCatalog, CompiledCollectible, CompiledCollection, CompiledConsole, CompiledGame,
};

/// Per-console game list under `database/games/*.json`.
#[derive(Debug, Deserialize)]
struct GameFile {
    console: GameFileConsoleHeader,
    #[serde(default)]
    games: Vec<RawGame>,
}

#[derive(Debug, Deserialize)]
struct GameFileConsoleHeader {
    id: String,
}

/// `database/consoles.json` — canonical console metadata.
#[derive(Debug, Deserialize)]
struct ConsolesFile {
    consoles: Vec<ConsoleListEntry>,
}

#[derive(Debug, Deserialize)]
struct ConsoleListEntry {
    id: String,
    name: String,
    manufacturer: String,
    #[serde(default)]
    family: String,
    #[serde(default)]
    form_factor: String,
    #[serde(default)]
    generation: Option<u32>,
    #[serde(default)]
    abbreviation: String,
}

#[derive(Debug, Deserialize)]
struct RawGame {
    title: String,
    slug: String,
    #[serde(default)]
    developer: Option<String>,
    #[serde(default)]
    publisher: Option<String>,
    #[serde(default)]
    first_release: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CollectibleFile {
    collection: CollectionHeader,
    #[serde(default)]
    items: Vec<RawCollectible>,
}

#[derive(Debug, Deserialize)]
struct CollectionHeader {
    id: String,
    name: String,
    #[serde(default)]
    manufacturer: String,
    #[serde(default, rename = "type")]
    kind: String,
}

#[derive(Debug, Deserialize)]
struct RawCollectible {
    name: String,
    slug: String,
    #[serde(default)]
    category: Option<String>,
    // Lego pack id.
    #[serde(default)]
    pack: Option<String>,
    // Skylander host game.
    #[serde(default)]
    game: Option<String>,
    // Skylander base color.
    #[serde(default)]
    base_color: Option<String>,
    // Lego year.
    #[serde(default)]
    year: Option<u8>,
}

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let db_dir = manifest_dir.join("..").join("..").join("database");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/compiled.rs");

    let (consoles, games) = load_consoles_and_games(&db_dir, &db_dir.join("games"));
    let (collections, collectibles) = load_collectibles(&db_dir.join("collectibles"));

    let catalog = CompiledCatalog {
        consoles,
        games,
        collections,
        collectibles,
    };

    let bytes = postcard::to_allocvec(&catalog).expect("serialize compiled catalog");
    fs::write(out_dir.join("catalog.postcard"), &bytes).expect("write catalog.postcard");
}

fn load_consoles_and_games(
    db_dir: &Path,
    games_dir: &Path,
) -> (Vec<CompiledConsole>, Vec<CompiledGame>) {
    let consoles_path = db_dir.join("consoles.json");
    println!("cargo:rerun-if-changed={}", consoles_path.display());

    let consoles_text = fs::read_to_string(&consoles_path).unwrap_or_else(|err| {
        panic!("read {}: {err}", consoles_path.display());
    });
    let consoles_file: ConsolesFile = serde_json::from_str(&consoles_text).unwrap_or_else(|err| {
        panic!("parse {}: {err}", consoles_path.display());
    });

    let mut consoles: Vec<CompiledConsole> = Vec::new();
    let mut known_short_ids: HashMap<String, String> = HashMap::new();

    for entry in &consoles_file.consoles {
        let console_id = format!("console:{}", entry.id);
        known_short_ids.insert(entry.id.clone(), console_id.clone());
        consoles.push(CompiledConsole {
            id: console_id,
            short_id: entry.id.clone(),
            name: entry.name.clone(),
            manufacturer: entry.manufacturer.clone(),
            family: entry.family.clone(),
            form_factor: entry.form_factor.clone(),
            generation: entry.generation.unwrap_or(0),
            abbreviation: entry.abbreviation.clone(),
        });
    }

    let mut games: Vec<CompiledGame> = Vec::new();
    let mut seen_slugs: HashMap<String, usize> = HashMap::new();

    let mut file_paths: Vec<PathBuf> = Vec::new();
    if games_dir.is_dir() {
        for entry in fs::read_dir(games_dir).unwrap_or_else(|err| {
            panic!("read {}: {err}", games_dir.display());
        }) {
            let entry = entry.expect("read games dir entry");
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(OsStr::to_str) != Some("json") {
                continue;
            }
            file_paths.push(path);
        }
    }
    file_paths.sort();

    for path in file_paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let text = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("read {}: {err}", path.display());
        });
        let parsed: GameFile = serde_json::from_str(&text).unwrap_or_else(|err| {
            panic!("parse {}: {err}", path.display());
        });

        let short_id = parsed.console.id.as_str();
        let console_id = known_short_ids.get(short_id).unwrap_or_else(|| {
            panic!(
                "{} references console id {:?} which is missing from database/consoles.json",
                path.display(),
                short_id
            )
        });

        for raw in parsed.games {
            let base = format!("{short_id}/{}", raw.slug);
            let count = seen_slugs.entry(base.clone()).or_insert(0);
            let final_slug = if *count == 0 {
                base.clone()
            } else {
                format!("{base}~{}", *count + 1)
            };
            *count += 1;

            games.push(CompiledGame {
                id: format!("game:{final_slug}"),
                console_id: console_id.clone(),
                console_short_id: short_id.to_string(),
                title: raw.title,
                developer: raw.developer.unwrap_or_default(),
                publisher: raw.publisher.unwrap_or_default(),
                year: parse_year(raw.first_release.as_deref()),
            });
        }
    }

    (consoles, games)
}

fn load_collectibles(dir: &Path) -> (Vec<CompiledCollection>, Vec<CompiledCollectible>) {
    let mut collections: Vec<CompiledCollection> = Vec::new();
    let mut collectibles: Vec<CompiledCollectible> = Vec::new();

    if !dir.exists() {
        return (collections, collectibles);
    }

    let mut file_paths: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir).expect("read collectibles directory") {
        let entry = entry.expect("read collectibles entry");
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(OsStr::to_str) != Some("json") {
            continue;
        }
        file_paths.push(path);
    }
    file_paths.sort();

    let mut seen_slugs: HashMap<String, usize> = HashMap::new();

    for path in file_paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let text = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("read {}: {err}", path.display());
        });
        let parsed: CollectibleFile = serde_json::from_str(&text).unwrap_or_else(|err| {
            panic!("parse {}: {err}", path.display());
        });

        collections.push(CompiledCollection {
            id: parsed.collection.id.clone(),
            name: parsed.collection.name,
            manufacturer: parsed.collection.manufacturer,
            kind: parsed.collection.kind,
        });

        for raw in parsed.items {
            let base = format!("{}/{}", parsed.collection.id, raw.slug);
            let count = seen_slugs.entry(base.clone()).or_insert(0);
            let final_slug = if *count == 0 {
                base.clone()
            } else {
                format!("{base}~{}", *count + 1)
            };
            *count += 1;

            collectibles.push(CompiledCollectible {
                id: format!("collectible:{final_slug}"),
                collection_id: parsed.collection.id.clone(),
                name: raw.name,
                category: raw.category.unwrap_or_default(),
                group: raw.pack.or(raw.game).unwrap_or_default(),
                variant: raw.base_color.unwrap_or_default(),
                year: raw.year.unwrap_or(0),
            });
        }
    }

    (collections, collectibles)
}

fn parse_year(date: Option<&str>) -> u32 {
    let Some(date) = date else { return 0 };
    date.split('-')
        .next()
        .and_then(|y| y.parse::<u32>().ok())
        .unwrap_or(0)
}
