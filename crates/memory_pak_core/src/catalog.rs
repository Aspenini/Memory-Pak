use crate::ids::{generate_stable_id, parse_year_from_iso_date};
use crate::models::{Catalog, Console, Game, LegoDimensionFigure, Skylander};
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use std::collections::HashMap;

static DATABASE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../database");
const CONSOLES_JSON: &str = include_str!("../../../database/consoles.json");
const LEGO_DIMENSIONS_JSON: &str = include_str!("../../../database/legodimensions.json");
const SKYLANDERS_JSON: &str = include_str!("../../../database/skylanders.json");

#[derive(Debug, Deserialize)]
struct DatabaseGame {
    title: String,
    publisher: String,
    #[serde(rename = "release_date")]
    release_date: Option<String>,
}

pub fn load_catalog() -> Catalog {
    Catalog {
        consoles: get_hardcoded_consoles(),
        games: load_embedded_games(),
        lego_dimensions_figures: load_lego_dimensions_figures(),
        skylanders: load_skylanders(),
    }
}

pub fn get_hardcoded_consoles() -> Vec<Console> {
    serde_json::from_str::<Vec<Console>>(CONSOLES_JSON).unwrap_or_default()
}

pub fn load_lego_dimensions_figures() -> Vec<LegoDimensionFigure> {
    serde_json::from_str::<Vec<LegoDimensionFigure>>(LEGO_DIMENSIONS_JSON).unwrap_or_default()
}

pub fn load_skylanders() -> Vec<Skylander> {
    serde_json::from_str::<Vec<Skylander>>(SKYLANDERS_JSON).unwrap_or_default()
}

pub fn load_embedded_games() -> HashMap<String, Game> {
    let mut games = HashMap::new();

    for file in DATABASE_DIR.files() {
        if file.path().extension().is_none_or(|ext| ext != "json") {
            continue;
        }

        let Some(file_stem) = file.path().file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        if matches!(file_stem, "consoles" | "legodimensions" | "skylanders") {
            continue;
        }

        let Some(console_id) = map_filename_to_console_id(file_stem) else {
            continue;
        };

        let Ok(content) = std::str::from_utf8(file.contents()) else {
            continue;
        };

        let Ok(db_games) = serde_json::from_str::<Vec<DatabaseGame>>(content) else {
            continue;
        };

        for db_game in db_games {
            let year = db_game
                .release_date
                .as_deref()
                .map(parse_year_from_iso_date)
                .unwrap_or(0);
            let game_id = generate_stable_id(
                console_id,
                &db_game.title,
                &db_game.publisher,
                db_game.release_date.as_deref(),
            );

            games.insert(
                game_id.clone(),
                Game {
                    id: game_id,
                    title: db_game.title,
                    year,
                    publisher: db_game.publisher,
                    console_id: console_id.to_string(),
                },
            );
        }
    }

    games
}

pub fn game_database_console_ids() -> Vec<String> {
    let mut ids: Vec<String> = DATABASE_DIR
        .files()
        .filter(|file| file.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|file| file.path().file_stem().and_then(|stem| stem.to_str()))
        .filter(|stem| !matches!(*stem, "consoles" | "legodimensions" | "skylanders"))
        .filter_map(map_filename_to_console_id)
        .map(str::to_string)
        .collect();

    ids.sort();
    ids.dedup();
    ids
}

pub fn map_filename_to_console_id(filename: &str) -> Option<&str> {
    match filename {
        "gameboy" => Some("gb"),
        "gameboycolor" => Some("gbc"),
        "gameboyadvance" => Some("gba"),
        "nintendods" => Some("ds"),
        "nintendo3ds" => Some("3ds"),
        "playstation" => Some("ps1"),
        "playstation2" => Some("ps2"),
        "playstation3" => Some("ps3"),
        "playstationvita" => Some("psvita"),
        _ => Some(filename),
    }
}

#[cfg(test)]
pub(crate) fn game_database_row_count() -> usize {
    DATABASE_DIR
        .files()
        .filter(|file| file.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|file| {
            file.path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| (stem, file))
        })
        .filter(|(stem, _)| !matches!(*stem, "consoles" | "legodimensions" | "skylanders"))
        .filter_map(|(_, file)| std::str::from_utf8(file.contents()).ok())
        .filter_map(|content| serde_json::from_str::<Vec<DatabaseGame>>(content).ok())
        .map(|games| games.len())
        .sum()
}
