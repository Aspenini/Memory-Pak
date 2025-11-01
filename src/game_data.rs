use crate::Game;
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

static GAMES_DIR: Dir = include_dir!("database");

#[derive(Deserialize)]
struct DatabaseGame {
    title: String,
    publisher: String,
    #[serde(rename = "release_date")]
    release_date: Option<String>,
}

fn map_filename_to_console_id(filename: &str) -> Option<&str> {
    match filename {
        "gameboy" => Some("gb"),
        "gameboycolor" => Some("gbc"),
        "gameboyadvance" => Some("gba"),
        "nintendods" => Some("ds"),
        "nintendo3ds" => Some("3ds"),
        "playstation" => Some("ps1"),
        "playstation2" => Some("ps2"),
        "playstation3" => Some("ps3"),
        _ => Some(filename), // Direct mapping for most consoles
    }
}

fn generate_stable_id(console_id: &str, title: &str) -> String {
    let mut hasher = DefaultHasher::new();
    console_id.hash(&mut hasher);
    title.hash(&mut hasher);
    format!("{}-{:x}", console_id, hasher.finish())
}

fn parse_year_from_iso_date(date_str: &str) -> u32 {
    date_str
        .split('-')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}

/// Extract console ID from a game's stable ID
/// Game IDs are in format: "console_id-hash"
pub fn get_console_from_id(game_id: &str) -> &str {
    game_id.split('-').next().unwrap_or("")
}

/// Load all games from the embedded JSON database
/// Returns a flat HashMap keyed by stable game ID
pub fn load_embedded_games() -> HashMap<String, Game> {
    let mut games = HashMap::new();

    for file in GAMES_DIR.files() {
        if let Some(ext) = file.path().extension() {
            if ext == "json" {
                if let Some(file_stem) = file.path().file_stem().and_then(|s| s.to_str()) {
                    if let Some(console_id) = map_filename_to_console_id(file_stem) {
                        if let Ok(content) = std::str::from_utf8(file.contents()) {
                            if let Ok(db_games) = serde_json::from_str::<Vec<DatabaseGame>>(content) {
                                for db_game in db_games {
                                    let year = db_game.release_date
                                        .as_ref()
                                        .map(|d| parse_year_from_iso_date(d))
                                        .unwrap_or(0);
                                    let game_id = generate_stable_id(console_id, &db_game.title);
                                    
                                    games.insert(game_id.clone(), Game {
                                        id: game_id,
                                        title: db_game.title,
                                        year,
                                        publisher: db_game.publisher,
                                        console_id: console_id.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    games
}
