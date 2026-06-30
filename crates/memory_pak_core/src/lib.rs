mod app;
mod catalog;
mod ids;
mod import_export;
mod model;
mod query;

pub use app::{CoreError, MemoryPakApp, SetItemNotesInput, SetItemStatusInput};
pub use catalog::catalog;
pub use ids::{normalize_for_search, EntryId, EntryKind};
pub use import_export::{
    apply_import, decode_persisted_state, encode_persisted_state, export_json_from_state,
    ExportData, ExportEntry, SaveDecodeError,
};
pub use memory_pak_catalog::{DatePrecision, PartialDate, RegionalReleases};
pub use model::{
    Catalog, CatalogRef, Collectible, CollectibleView, Collection, CollectionStats, CollectionView,
    Console, ConsoleCounts, ConsoleView, EntryOverride, EntryState, Game, GameView, InitialState,
    ItemKind, MutationResult, PersistedState, QuerySnapshot, RowView, SaveEnvelopeV3,
    SAVE_SCHEMA_VERSION,
};
pub use query::{FilterBy, QueryInput, QueryResult, QuerySpec, SortKey};

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
            assert!(view.game_counts.total > 0);
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

    #[test]
    fn legacy_save_migrates_to_sparse_v3_and_preserves_unknown_ids() {
        let legacy = r#"{
          "entries": {
            "game:nes/super-mario-bros": {
              "owned": true,
              "favorite": false,
              "wishlist": false,
              "notes": ""
            },
            "game:removed/temporarily-missing": {
              "favorite": true,
              "notes": "keep this"
            },
            "console:empty": {
              "owned": false,
              "favorite": false,
              "wishlist": false,
              "notes": ""
            }
          }
        }"#;

        let app = MemoryPakApp::from_save_json(legacy).expect("legacy save");
        let json = app.save_json().expect("v3 save");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["schemaVersion"], 3);
        assert_eq!(
            value["entries"]["game:removed/temporarily-missing"]["notes"],
            "keep this"
        );
        assert!(value["entries"].get("console:empty").is_none());
        assert!(value["entries"]["game:nes/super-mario-bros"]
            .get("favorite")
            .is_none());
        assert_eq!(app.collection_stats().favorite_games, 0);
    }

    #[test]
    fn indexed_game_search_matches_a_simple_title_scan() {
        let app = MemoryPakApp::default();
        for search in ["mario", "Pokémon", "Final Fantasy", "x", "zzzz-not-found"] {
            let normalized = normalize_for_search(search);
            let mut expected: Vec<&str> = app
                .catalog()
                .games
                .values()
                .filter(|game| normalize_for_search(&game.title).contains(&normalized))
                .map(|game| game.id.as_str())
                .collect();
            expected.sort_by(|left, right| {
                let left_game = app.catalog().games.get(*left).unwrap();
                let right_game = app.catalog().games.get(*right).unwrap();
                left_game
                    .title
                    .cmp(&right_game.title)
                    .then_with(|| left.cmp(right))
            });

            let result = app.query_snapshot(QuerySpec {
                kind: ItemKind::Game,
                search: Some(search.to_string()),
                sort_by: Some(SortKey::Title),
                ..Default::default()
            });
            let actual: Vec<String> = result
                .items
                .into_iter()
                .map(|reference| match app.row_view(reference).unwrap() {
                    RowView::Game(game) => game.id.into_string(),
                    _ => unreachable!(),
                })
                .filter(|id| {
                    normalize_for_search(&app.catalog().games.get(id.as_str()).unwrap().title)
                        .contains(&normalized)
                })
                .collect();
            let expected: Vec<&str> = expected
                .into_iter()
                .filter(|id| {
                    normalize_for_search(&app.catalog().games.get(*id).unwrap().title)
                        .contains(&normalized)
                })
                .collect();

            assert_eq!(actual, expected, "search {search:?}");
        }
    }

    #[test]
    fn collectible_suffix_ids_are_stable() {
        let catalog = catalog();
        assert!(catalog
            .collectibles
            .iter()
            .any(|item| item.id.as_str().contains("~2")));
        assert!(catalog
            .collectibles
            .iter()
            .any(|item| item.id.as_str().contains("~3")));
    }

    #[test]
    fn indexed_filter_sort_group_and_pagination_match_reference() {
        let catalog = catalog();
        let mut games: Vec<_> = catalog.games.values().cloned().collect();
        games.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        let mut state = PersistedState::default();
        for (index, game) in games.iter().take(80).enumerate() {
            state.entries.insert(
                game.id.clone(),
                EntryState {
                    owned: index % 2 == 0,
                    favorite: index % 3 == 0,
                    wishlist: index % 5 == 0,
                    notes: String::new(),
                },
            );
        }
        let app = MemoryPakApp::from_persisted_state(state);
        let group = games[0].console_id.to_string();
        let cases = [
            (FilterBy::Owned, SortKey::Status, "all", 3, 17),
            (FilterBy::Favorites, SortKey::Year, "all", 0, 13),
            (FilterBy::NotOwned, SortKey::Title, group.as_str(), 2, 11),
            (FilterBy::Wishlist, SortKey::Status, group.as_str(), 0, 19),
        ];

        for (filter, sort, group_id, offset, limit) in cases {
            let mut expected: Vec<_> = games
                .iter()
                .filter(|game| group_id == "all" || game.console_id.as_str() == group_id)
                .filter(|game| {
                    let state = app
                        .persisted_state()
                        .entries
                        .get(&game.id)
                        .cloned()
                        .unwrap_or_default();
                    match filter {
                        FilterBy::All => true,
                        FilterBy::Owned => state.owned,
                        FilterBy::Favorites => state.favorite,
                        FilterBy::Wishlist => state.wishlist,
                        FilterBy::NotOwned => !state.owned,
                    }
                })
                .collect();
            expected.sort_by(|left, right| {
                let title_order = left
                    .title
                    .cmp(&right.title)
                    .then_with(|| left.id.as_str().cmp(right.id.as_str()));
                match sort {
                    SortKey::Year => left.year.cmp(&right.year).then(title_order),
                    SortKey::Status => {
                        let score = |game: &Game| {
                            let state = app
                                .persisted_state()
                                .entries
                                .get(&game.id)
                                .cloned()
                                .unwrap_or_default();
                            if state.owned {
                                3
                            } else if state.favorite {
                                2
                            } else if state.wishlist {
                                1
                            } else {
                                0
                            }
                        };
                        score(right).cmp(&score(left)).then(title_order)
                    }
                    _ => title_order,
                }
            });
            let total = expected.len();
            let expected: Vec<&str> = expected
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|game| game.id.as_str())
                .collect();

            let result = app.query_snapshot(QuerySpec {
                kind: ItemKind::Game,
                sort_by: Some(sort),
                filter_by: Some(filter),
                group_id: Some(group_id.to_string()),
                offset: Some(offset),
                limit: Some(limit),
                ..Default::default()
            });
            let actual: Vec<String> = result
                .items
                .into_iter()
                .map(|reference| match app.row_view(reference).unwrap() {
                    RowView::Game(game) => game.id.into_string(),
                    _ => unreachable!(),
                })
                .collect();

            assert_eq!(result.total, total);
            assert_eq!(actual, expected);
        }
    }
}
