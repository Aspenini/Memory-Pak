use crate::Game;
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;

static GAMES_DIR: Dir = include_dir!("database");

#[derive(Debug, Deserialize)]
struct DatabaseGame {
    title: String,
    #[allow(dead_code)]
    developer: String,
    publisher: String,
    #[serde(rename = "first_released")]
    first_released: Option<String>,
    #[allow(dead_code)]
    category: String,
}

// Map database filenames to console IDs used in the app
fn map_filename_to_console_id(filename: &str) -> Option<&str> {
    match filename {
        "nes_all" => Some("nes"),
        "snes_all" => Some("snes"),
        "n64_all" => Some("n64"),
        "gamecube_all" => Some("gamecube"),
        "wii_all" => Some("wii"),
        "wiiu_all" => Some("wiiu"),
        "gameboy_all" => Some("gb"),
        "gameboycolor_all" => Some("gbc"),
        "gameboyadvance_all" => Some("gba"),
        "nintendods_all" => Some("ds"),
        "nintendo3ds_all" => Some("3ds"),
        "genesis_all" => Some("genesis"),
        "mastersystem_all" => Some("mastersystem"),
        "saturn_all" => Some("saturn"),
        "dreamcast_all" => Some("dreamcast"),
        "sg1000_all" => Some("sg1000"),
        "segacd_all" => Some("segacd"),
        "sega32x_all" => Some("sega32x"),
        "gamegear_all" => Some("gamegear"),
        "pico_all" => Some("pico"),
        "playstation_all" => Some("ps1"),
        "playstation2_all" => Some("ps2"),
        "playstation3_all" => Some("ps3"),
        "psp_all" => Some("psp"),
        "playstationvita_all" => Some("psvita"),
        _ => None,
    }
}

fn parse_year_from_date(date_str: &str) -> u32 {
    // Try to extract year from date strings like "December 10, 1988", "September 1987", or "1988"
    // First try to find a 4-digit number
    if let Some(year) = date_str
        .split_whitespace()
        .flat_map(|s| s.trim_matches(|c: char| !c.is_ascii_digit()).parse::<u32>().ok())
        .find(|&y| y >= 1970 && y <= 2100)
    {
        return year;
    }
    
    // Fallback: try splitting by comma and getting last part
    date_str
        .split(',')
        .last()
        .and_then(|s| s.trim().split_whitespace().last())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}

pub fn load_embedded_games() -> HashMap<String, Vec<Game>> {
    let mut games: HashMap<String, Vec<Game>> = HashMap::new();

    for file in GAMES_DIR.files() {
        if let Some(file_name) = file.path().file_stem().and_then(|s| s.to_str()) {
            // Map database filename to console ID
            if let Some(console_id) = map_filename_to_console_id(file_name) {
                if let Ok(json_str) = std::str::from_utf8(file.contents()) {
                    if let Ok(db_games) = serde_json::from_str::<Vec<DatabaseGame>>(json_str) {
                        let games_list: Vec<Game> = db_games
                            .into_iter()
                            .enumerate()
                            .map(|(idx, db_game)| {
                                let year = db_game.first_released
                                    .as_ref()
                                    .map(|d| parse_year_from_date(d))
                                    .unwrap_or(0);
                                // Create a more stable game ID using a hash of the title
                                let game_id = format!("{}-{}", console_id, idx);
                                Game {
                                    id: game_id,
                                    title: db_game.title,
                                    year,
                                    publisher: db_game.publisher,
                                }
                            })
                            .collect();
                        games.insert(console_id.to_string(), games_list);
                    }
                }
            }
        }
    }

    games
}
