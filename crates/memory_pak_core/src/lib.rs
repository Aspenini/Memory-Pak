mod app;
mod catalog;
mod ids;
mod import_export;
mod models;
mod query;

pub use app::{CoreError, MemoryPakApp, SetItemNotesInput, SetItemStatusInput};
pub use catalog::{
    game_database_console_ids, get_hardcoded_consoles, load_catalog, load_embedded_games,
    load_lego_dimensions_figures, load_skylanders, map_filename_to_console_id,
};
pub use ids::{
    figure_id, generate_legacy_id, generate_stable_id, get_console_from_id, skylander_id,
};
pub use import_export::{export_json_from_state, ConsoleExportData, ExportData};
pub use models::{
    Catalog, CollectionStats, Console, ConsoleCounts, ConsoleState, ConsoleView, Game, GameState,
    GameView, InitialState, ItemKind, LegoDimensionFigure, LegoDimensionState, LegoView,
    PersistedState, Skylander, SkylanderState, SkylanderView, StatusState,
};
pub use query::{QueryInput, QueryResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::game_database_row_count;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn generated_game_ids_are_unique_for_database_rows() {
        let games = load_embedded_games();
        assert_eq!(
            games.len(),
            game_database_row_count(),
            "every database row should produce a unique game ID"
        );
    }

    #[test]
    fn game_database_console_ids_have_console_metadata() {
        let console_ids: HashSet<String> = get_hardcoded_consoles()
            .into_iter()
            .map(|console| console.id)
            .collect();

        for console_id in game_database_console_ids() {
            assert!(
                console_ids.contains(&console_id),
                "game database console ID {console_id:?} is missing from consoles.json"
            );
        }
    }

    #[test]
    fn filename_aliases_match_saved_state_console_ids() {
        assert_eq!(
            map_filename_to_console_id("playstationvita"),
            Some("psvita")
        );
        assert_eq!(map_filename_to_console_id("nintendods"), Some("ds"));
        assert_eq!(map_filename_to_console_id("gameboyadvance"), Some("gba"));
    }

    #[test]
    fn console_id_extraction_allows_hyphenated_console_ids() {
        assert_eq!(
            get_console_from_id("pc-engine-0123456789abcdef"),
            "pc-engine"
        );
        assert_eq!(get_console_from_id("psvita-0123456789abcdef"), "psvita");
        assert_eq!(get_console_from_id("malformed"), "");
    }

    #[test]
    fn export_json_has_stable_ordering() {
        let state = PersistedState {
            console_states: HashMap::from([
                (
                    "snes".to_string(),
                    ConsoleState {
                        console_id: "snes".to_string(),
                        owned: true,
                        ..Default::default()
                    },
                ),
                (
                    "nes".to_string(),
                    ConsoleState {
                        console_id: "nes".to_string(),
                        wishlist: true,
                        ..Default::default()
                    },
                ),
            ]),
            game_states: HashMap::from([
                (
                    "snes-bbbbbbbbbbbbbbbb".to_string(),
                    GameState {
                        game_id: "snes-bbbbbbbbbbbbbbbb".to_string(),
                        owned: true,
                        ..Default::default()
                    },
                ),
                (
                    "nes-aaaaaaaaaaaaaaaa".to_string(),
                    GameState {
                        game_id: "nes-aaaaaaaaaaaaaaaa".to_string(),
                        favorite: true,
                        ..Default::default()
                    },
                ),
            ]),
            lego_dimensions_states: HashMap::from([(
                "lego-b".to_string(),
                LegoDimensionState {
                    figure_id: "lego-b".to_string(),
                    owned: true,
                    ..Default::default()
                },
            )]),
            skylanders_states: HashMap::from([(
                "skylander-a".to_string(),
                SkylanderState {
                    skylander_id: "skylander-a".to_string(),
                    wishlist: true,
                    ..Default::default()
                },
            )]),
        };

        let first = export_json_from_state(&state).expect("export should serialize");
        let second = export_json_from_state(&state).expect("export should serialize");

        let export: ExportData = serde_json::from_str(&first).expect("export should parse");
        let second_export: ExportData = serde_json::from_str(&second).expect("export should parse");
        assert_eq!(
            export
                .console_states
                .iter()
                .map(|state| state.console_id.as_str())
                .collect::<Vec<_>>(),
            second_export
                .console_states
                .iter()
                .map(|state| state.console_id.as_str())
                .collect::<Vec<_>>()
        );
        assert_eq!(export.console_states[0].console_id, "nes");
        assert_eq!(export.console_states[1].console_id, "snes");
        assert_eq!(export.consoles[0].console_id, "nes");
        assert_eq!(export.consoles[1].console_id, "snes");
    }

    #[test]
    fn state_mutations_update_status_notes_and_counts() {
        let mut app = MemoryPakApp::default();
        let game = app
            .query_games(QueryInput {
                search: Some("Mario".to_string()),
                limit: Some(1),
                ..Default::default()
            })
            .items
            .remove(0);

        app.set_item_status(SetItemStatusInput {
            kind: ItemKind::Game,
            id: game.id.clone(),
            owned: Some(true),
            favorite: Some(true),
            wishlist: None,
        })
        .expect("game status should update");
        app.set_item_notes(SetItemNotesInput {
            kind: ItemKind::Game,
            id: game.id.clone(),
            notes: "boxed".to_string(),
        })
        .expect("game notes should update");

        let updated = app
            .query_games(QueryInput {
                search: Some(game.title),
                limit: Some(1),
                ..Default::default()
            })
            .items
            .remove(0);
        assert!(updated.state.owned);
        assert!(updated.state.favorite);
        assert_eq!(updated.state.notes, "boxed");
        assert_eq!(app.collection_stats().owned_games, 1);
    }

    #[test]
    fn import_merges_all_supported_state_groups() {
        let export = ExportData {
            version: "1.0".to_string(),
            export_date: "2024-01-01T00:00:00Z".to_string(),
            console_states: vec![ConsoleState {
                console_id: "nes".to_string(),
                owned: true,
                ..Default::default()
            }],
            consoles: vec![ConsoleExportData {
                console_id: "nes".to_string(),
                games: vec![GameState {
                    game_id: "nes-aaaaaaaaaaaaaaaa".to_string(),
                    wishlist: true,
                    ..Default::default()
                }],
            }],
            lego_dimensions_states: vec![LegoDimensionState {
                figure_id: "lego-batman-batman".to_string(),
                owned: true,
                ..Default::default()
            }],
            skylanders_states: vec![SkylanderState {
                skylander_id: "skylander-spyros-adventure-spyro-core".to_string(),
                favorite: true,
                ..Default::default()
            }],
        };
        let json = serde_json::to_string(&export).expect("fixture should serialize");
        let mut app = MemoryPakApp::default();

        app.import_json(&json)
            .expect("import should parse and merge");

        assert!(app.persisted_state().console_states["nes"].owned);
        assert!(
            app.persisted_state()
                .game_states
                .get("nes-aaaaaaaaaaaaaaaa")
                .expect("game state should be imported")
                .wishlist
        );
    }

    #[test]
    fn malformed_import_is_reported() {
        let mut app = MemoryPakApp::default();
        assert!(matches!(
            app.import_json("{ nope"),
            Err(CoreError::InvalidImport(_))
        ));
    }
}
