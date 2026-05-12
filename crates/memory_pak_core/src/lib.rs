mod app;
mod catalog;
mod compiled;
mod ids;
mod import_export;
mod model;
mod query;

pub use app::{CoreError, MemoryPakApp, SetItemNotesInput, SetItemStatusInput};
pub use catalog::catalog;
pub use ids::{normalize_for_search, EntryId, EntryKind};
pub use import_export::{apply_import, export_json_from_state, ExportData, ExportEntry};
pub use model::{
    Catalog, Collectible, CollectibleView, Collection, CollectionStats, CollectionView, Console,
    ConsoleCounts, ConsoleView, EntryState, Game, GameView, InitialState, ItemKind, MutationResult,
    PersistedState,
};
pub use query::{FilterBy, QueryInput, QueryResult, SortKey};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_loads_with_unique_game_ids() {
        let cat = catalog();
        assert!(!cat.consoles.is_empty());
        assert!(!cat.games.is_empty());
        let unique: std::collections::HashSet<&EntryId> = cat.games.keys().collect();
        assert_eq!(unique.len(), cat.games.len());
        for game in cat.games.values() {
            assert_eq!(game.id.kind(), Some(EntryKind::Game));
            assert_eq!(game.console_id.kind(), Some(EntryKind::Console));
        }
    }

    #[test]
    fn collectibles_belong_to_known_collections() {
        let cat = catalog();
        let known: std::collections::HashSet<&str> =
            cat.collections.iter().map(|c| c.id.as_str()).collect();
        assert!(!cat.collectibles.is_empty());
        for item in &cat.collectibles {
            assert!(known.contains(item.collection_id.as_str()));
            assert_eq!(item.id.kind(), Some(EntryKind::Collectible));
        }
    }

    #[test]
    fn console_ids_are_unique() {
        let cat = catalog();
        let unique: std::collections::HashSet<&EntryId> =
            cat.consoles.iter().map(|c| &c.id).collect();
        assert_eq!(unique.len(), cat.consoles.len());
        for console in &cat.consoles {
            assert_eq!(console.id.kind(), Some(EntryKind::Console));
        }
    }

    #[test]
    fn export_is_deterministic_and_sorted() {
        let mut app = MemoryPakApp::default();
        let mario = app
            .query_games(QueryInput {
                search: Some("Mario".to_string()),
                limit: Some(1),
                ..Default::default()
            })
            .items
            .remove(0);

        app.set_item_status(SetItemStatusInput {
            id: mario.id.clone(),
            owned: Some(true),
            favorite: None,
            wishlist: None,
        })
        .expect("status update");

        let first = app.export_json().expect("export");
        let second = app.export_json().expect("export");
        let parsed_a: ExportData = serde_json::from_str(&first).unwrap();
        let parsed_b: ExportData = serde_json::from_str(&second).unwrap();
        assert_eq!(parsed_a.entries, parsed_b.entries);
        assert_eq!(parsed_a.version, "2.0");
        assert_eq!(parsed_a.entries.len(), 1);
        assert_eq!(parsed_a.entries[0].id, mario.id);
    }

    #[test]
    fn status_mutation_returns_delta_and_updates_stats() {
        let mut app = MemoryPakApp::default();
        let game = app
            .query_games(QueryInput {
                search: Some("Mario".to_string()),
                limit: Some(1),
                ..Default::default()
            })
            .items
            .remove(0);

        let result = app
            .set_item_status(SetItemStatusInput {
                id: game.id.clone(),
                owned: Some(true),
                favorite: Some(true),
                wishlist: None,
            })
            .expect("status update");

        assert_eq!(result.id, game.id);
        assert!(result.state.owned);
        assert!(result.state.favorite);
        assert_eq!(result.stats.owned_games, 1);
        assert_eq!(result.stats.favorite_games, 1);
    }

    #[test]
    fn empty_state_is_pruned() {
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
            id: game.id.clone(),
            owned: Some(true),
            favorite: None,
            wishlist: None,
        })
        .expect("status update");
        assert!(app.persisted_state().entries.contains_key(&game.id));

        app.set_item_status(SetItemStatusInput {
            id: game.id.clone(),
            owned: Some(false),
            favorite: None,
            wishlist: None,
        })
        .expect("status update");
        assert!(!app.persisted_state().entries.contains_key(&game.id));
    }

    #[test]
    fn import_merges_state_and_rejects_garbage() {
        let mut app = MemoryPakApp::default();
        let game = app
            .query_games(QueryInput {
                search: Some("Mario".to_string()),
                limit: Some(1),
                ..Default::default()
            })
            .items
            .remove(0);

        let export = ExportData {
            version: "2.0".to_string(),
            exported_at: "2024-01-01T00:00:00Z".to_string(),
            entries: vec![ExportEntry {
                id: game.id.clone(),
                owned: true,
                favorite: false,
                wishlist: false,
                notes: "cart only".to_string(),
            }],
        };
        let json = serde_json::to_string(&export).unwrap();

        app.import_json(&json).expect("import");
        assert_eq!(
            app.persisted_state()
                .entries
                .get(&game.id)
                .map(|s| s.notes.as_str()),
            Some("cart only")
        );

        assert!(matches!(
            app.import_json("{ nope"),
            Err(CoreError::InvalidImport(_))
        ));
    }

    #[test]
    fn consoles_with_games_matches_catalog() {
        let app = MemoryPakApp::default();
        let init = app.initial_state();
        let expected: std::collections::HashSet<_> = app
            .catalog()
            .games
            .values()
            .map(|g| g.console_id.clone())
            .collect();

        assert!(init.consoles_with_games.len() <= init.consoles.len());
        for view in &init.consoles_with_games {
            assert!(expected.contains(&view.id));
        }
        for console in &init.consoles {
            if expected.contains(&console.id) {
                assert!(init.consoles_with_games.iter().any(|v| v.id == console.id));
            }
        }
    }

    #[test]
    fn collectible_query_filters_by_collection() {
        let app = MemoryPakApp::default();
        let collections = app.initial_state().collections;
        if let Some(first) = collections.first() {
            let result = app.query_collectibles(QueryInput {
                collection_id: Some(first.id.clone()),
                ..Default::default()
            });
            for item in &result.items {
                assert_eq!(item.collection_id, first.id);
            }
        }
    }
}
