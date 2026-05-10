use crate::catalog::load_catalog;
use crate::ids::{figure_id, generate_legacy_id, get_console_from_id, skylander_id};
use crate::import_export::{export_json_from_state, ExportData};
use crate::models::{
    Catalog, CollectionStats, Console, ConsoleCounts, ConsoleState, ConsoleView, Game, GameState,
    GameView, InitialState, ItemKind, LegoDimensionFigure, LegoDimensionState, LegoView,
    PersistedState, Skylander, SkylanderState, SkylanderView, StatusState,
};
use crate::query::{
    normalize_for_search, normalized_filter, normalized_query, paginate, status_matches,
    status_score, QueryInput, QueryResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid import JSON: {0}")]
    InvalidImport(#[from] serde_json::Error),
    #[error("unknown item {kind:?}:{id}")]
    UnknownItem { kind: ItemKind, id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetItemStatusInput {
    pub kind: ItemKind,
    pub id: String,
    #[serde(default)]
    pub owned: Option<bool>,
    #[serde(default)]
    pub favorite: Option<bool>,
    #[serde(default)]
    pub wishlist: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetItemNotesInput {
    pub kind: ItemKind,
    pub id: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct MemoryPakApp {
    catalog: Catalog,
    state: PersistedState,
    game_counts_by_console: HashMap<String, ConsoleCounts>,
}

impl Default for MemoryPakApp {
    fn default() -> Self {
        Self::from_persisted_state(PersistedState::default())
    }
}

impl MemoryPakApp {
    pub fn from_persisted_state(state: PersistedState) -> Self {
        let catalog = load_catalog();
        let mut app = Self {
            catalog,
            state,
            game_counts_by_console: HashMap::new(),
        };
        app.migrate_legacy_game_state_ids();
        app.refresh_game_counts();
        app
    }

    pub fn persisted_state(&self) -> PersistedState {
        self.state.clone()
    }

    pub fn initial_state(&self) -> InitialState {
        InitialState {
            stats: self.collection_stats(),
            consoles: self.query_consoles(QueryInput::default()).items,
            total_games: self.catalog.games.len(),
            total_lego_dimensions: self.catalog.lego_dimensions_figures.len(),
            total_skylanders: self.catalog.skylanders.len(),
        }
    }

    pub fn query_consoles(&self, input: QueryInput) -> QueryResult<ConsoleView> {
        let search = normalized_query(input.search.as_deref());
        let filter = normalized_filter(input.filter_by.as_deref());
        let sort = input.sort_by.unwrap_or_else(|| "title".to_string());

        let mut items: Vec<ConsoleView> = self
            .catalog
            .consoles
            .iter()
            .filter(|console| {
                search.as_ref().is_none_or(|query| {
                    normalize_for_search(&console.name).contains(query)
                        || normalize_for_search(&console.manufacturer).contains(query)
                })
            })
            .map(|console| self.console_view(console))
            .filter(|view| status_matches(&view.state, filter))
            .collect();

        items.sort_by(|a, b| match sort.as_str() {
            "year" => a.year.cmp(&b.year).then_with(|| a.name.cmp(&b.name)),
            "status" => status_score(&b.state)
                .cmp(&status_score(&a.state))
                .then_with(|| a.name.cmp(&b.name)),
            _ => a.name.cmp(&b.name),
        });

        paginate(items, input.offset, input.limit)
    }

    pub fn query_games(&self, input: QueryInput) -> QueryResult<GameView> {
        let search = normalized_query(input.search.as_deref());
        let filter = normalized_filter(input.filter_by.as_deref());
        let sort = input.sort_by.unwrap_or_else(|| "title".to_string());
        let console_filter = input.console_id.as_deref().unwrap_or("all");
        let console_names = self.console_names();

        let mut items: Vec<GameView> = self
            .catalog
            .games
            .values()
            .filter(|game| console_filter == "all" || console_filter == game.console_id)
            .filter(|game| {
                search.as_ref().is_none_or(|query| {
                    normalize_for_search(&game.title).contains(query)
                        || normalize_for_search(&game.publisher).contains(query)
                        || console_names
                            .get(&game.console_id)
                            .map(|name| normalize_for_search(name).contains(query))
                            .unwrap_or(false)
                })
            })
            .map(|game| self.game_view(game, &console_names))
            .filter(|view| status_matches(&view.state, filter))
            .collect();

        items.sort_by(|a, b| match sort.as_str() {
            "year" => a.year.cmp(&b.year).then_with(|| a.title.cmp(&b.title)),
            "status" => status_score(&b.state)
                .cmp(&status_score(&a.state))
                .then_with(|| a.title.cmp(&b.title)),
            _ => a.title.cmp(&b.title),
        });

        paginate(items, input.offset, input.limit)
    }

    pub fn query_lego(&self, input: QueryInput) -> QueryResult<LegoView> {
        let search = normalized_query(input.search.as_deref());
        let filter = normalized_filter(input.filter_by.as_deref());
        let sort = input.sort_by.unwrap_or_else(|| "name".to_string());

        let mut items: Vec<LegoView> = self
            .catalog
            .lego_dimensions_figures
            .iter()
            .filter(|figure| {
                search.as_ref().is_none_or(|query| {
                    normalize_for_search(&figure.name).contains(query)
                        || normalize_for_search(&figure.category).contains(query)
                        || normalize_for_search(&figure.pack_id).contains(query)
                })
            })
            .map(|figure| self.lego_view(figure))
            .filter(|view| status_matches(&view.state, filter))
            .collect();

        items.sort_by(|a, b| match sort.as_str() {
            "category" => a
                .category
                .cmp(&b.category)
                .then_with(|| a.name.cmp(&b.name)),
            "year" => a.year.cmp(&b.year).then_with(|| a.name.cmp(&b.name)),
            "pack" | "packId" => a.pack_id.cmp(&b.pack_id).then_with(|| a.name.cmp(&b.name)),
            "status" => status_score(&b.state)
                .cmp(&status_score(&a.state))
                .then_with(|| a.name.cmp(&b.name)),
            _ => a.name.cmp(&b.name),
        });

        paginate(items, input.offset, input.limit)
    }

    pub fn query_skylanders(&self, input: QueryInput) -> QueryResult<SkylanderView> {
        let search = normalized_query(input.search.as_deref());
        let filter = normalized_filter(input.filter_by.as_deref());
        let sort = input.sort_by.unwrap_or_else(|| "name".to_string());

        let mut items: Vec<SkylanderView> = self
            .catalog
            .skylanders
            .iter()
            .filter(|skylander| {
                search.as_ref().is_none_or(|query| {
                    normalize_for_search(&skylander.name).contains(query)
                        || normalize_for_search(&skylander.game).contains(query)
                        || normalize_for_search(&skylander.base_color).contains(query)
                        || normalize_for_search(&skylander.category).contains(query)
                })
            })
            .map(|skylander| self.skylander_view(skylander))
            .filter(|view| status_matches(&view.state, filter))
            .collect();

        items.sort_by(|a, b| match sort.as_str() {
            "game" => a.game.cmp(&b.game).then_with(|| a.name.cmp(&b.name)),
            "baseColor" | "base_color" => a
                .base_color
                .cmp(&b.base_color)
                .then_with(|| a.name.cmp(&b.name)),
            "category" => a
                .category
                .cmp(&b.category)
                .then_with(|| a.name.cmp(&b.name)),
            "status" => status_score(&b.state)
                .cmp(&status_score(&a.state))
                .then_with(|| a.name.cmp(&b.name)),
            _ => a.name.cmp(&b.name),
        });

        paginate(items, input.offset, input.limit)
    }

    pub fn set_item_status(
        &mut self,
        input: SetItemStatusInput,
    ) -> Result<PersistedState, CoreError> {
        match input.kind {
            ItemKind::Console => {
                self.ensure_console(&input.id)?;
                let state = self
                    .state
                    .console_states
                    .entry(input.id.clone())
                    .or_insert_with(|| ConsoleState {
                        console_id: input.id,
                        ..Default::default()
                    });
                apply_status_update(
                    &mut state.owned,
                    &mut state.favorite,
                    &mut state.wishlist,
                    input.owned,
                    input.favorite,
                    input.wishlist,
                );
            }
            ItemKind::Game => {
                self.ensure_game(&input.id)?;
                let state = self
                    .state
                    .game_states
                    .entry(input.id.clone())
                    .or_insert_with(|| GameState {
                        game_id: input.id,
                        ..Default::default()
                    });
                apply_status_update(
                    &mut state.owned,
                    &mut state.favorite,
                    &mut state.wishlist,
                    input.owned,
                    input.favorite,
                    input.wishlist,
                );
                self.refresh_game_counts();
            }
            ItemKind::Lego => {
                self.ensure_lego(&input.id)?;
                let state = self
                    .state
                    .lego_dimensions_states
                    .entry(input.id.clone())
                    .or_insert_with(|| LegoDimensionState {
                        figure_id: input.id,
                        ..Default::default()
                    });
                apply_status_update(
                    &mut state.owned,
                    &mut state.favorite,
                    &mut state.wishlist,
                    input.owned,
                    input.favorite,
                    input.wishlist,
                );
            }
            ItemKind::Skylander => {
                self.ensure_skylander(&input.id)?;
                let state = self
                    .state
                    .skylanders_states
                    .entry(input.id.clone())
                    .or_insert_with(|| SkylanderState {
                        skylander_id: input.id,
                        ..Default::default()
                    });
                apply_status_update(
                    &mut state.owned,
                    &mut state.favorite,
                    &mut state.wishlist,
                    input.owned,
                    input.favorite,
                    input.wishlist,
                );
            }
        }

        Ok(self.persisted_state())
    }

    pub fn set_item_notes(
        &mut self,
        input: SetItemNotesInput,
    ) -> Result<PersistedState, CoreError> {
        match input.kind {
            ItemKind::Console => {
                self.ensure_console(&input.id)?;
                self.state
                    .console_states
                    .entry(input.id.clone())
                    .or_insert_with(|| ConsoleState {
                        console_id: input.id.clone(),
                        ..Default::default()
                    })
                    .notes = input.notes;
            }
            ItemKind::Game => {
                self.ensure_game(&input.id)?;
                self.state
                    .game_states
                    .entry(input.id.clone())
                    .or_insert_with(|| GameState {
                        game_id: input.id.clone(),
                        ..Default::default()
                    })
                    .notes = input.notes;
            }
            ItemKind::Lego => {
                self.ensure_lego(&input.id)?;
                self.state
                    .lego_dimensions_states
                    .entry(input.id.clone())
                    .or_insert_with(|| LegoDimensionState {
                        figure_id: input.id.clone(),
                        ..Default::default()
                    })
                    .notes = input.notes;
            }
            ItemKind::Skylander => {
                self.ensure_skylander(&input.id)?;
                self.state
                    .skylanders_states
                    .entry(input.id.clone())
                    .or_insert_with(|| SkylanderState {
                        skylander_id: input.id.clone(),
                        ..Default::default()
                    })
                    .notes = input.notes;
            }
        }

        Ok(self.persisted_state())
    }

    pub fn import_json(&mut self, json: &str) -> Result<PersistedState, CoreError> {
        let import = serde_json::from_str::<ExportData>(json)?;
        self.apply_import(import);
        Ok(self.persisted_state())
    }

    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        export_json_from_state(&self.state)
    }

    pub fn collection_stats(&self) -> CollectionStats {
        let owned_consoles = self
            .state
            .console_states
            .values()
            .filter(|state| state.owned)
            .count();
        let favorite_consoles = self
            .state
            .console_states
            .values()
            .filter(|state| state.favorite)
            .count();
        let wishlist_consoles = self
            .state
            .console_states
            .values()
            .filter(|state| state.wishlist)
            .count();
        let owned_games = self
            .state
            .game_states
            .values()
            .filter(|state| state.owned)
            .count();
        let favorite_games = self
            .state
            .game_states
            .values()
            .filter(|state| state.favorite)
            .count();
        let wishlist_games = self
            .state
            .game_states
            .values()
            .filter(|state| state.wishlist)
            .count();
        let owned_lego_dimensions = self
            .state
            .lego_dimensions_states
            .values()
            .filter(|state| state.owned)
            .count();
        let owned_skylanders = self
            .state
            .skylanders_states
            .values()
            .filter(|state| state.owned)
            .count();

        CollectionStats {
            total_consoles: self.catalog.consoles.len(),
            owned_consoles,
            favorite_consoles,
            wishlist_consoles,
            total_games: self.catalog.games.len(),
            owned_games,
            favorite_games,
            wishlist_games,
            total_lego_dimensions: self.catalog.lego_dimensions_figures.len(),
            owned_lego_dimensions,
            total_skylanders: self.catalog.skylanders.len(),
            owned_skylanders,
        }
    }

    fn apply_import(&mut self, import: ExportData) {
        for state in import.console_states {
            self.state
                .console_states
                .insert(state.console_id.clone(), state);
        }

        for console_export in import.consoles {
            for state in console_export.games {
                self.state.game_states.insert(state.game_id.clone(), state);
            }
        }

        for state in import.lego_dimensions_states {
            self.state
                .lego_dimensions_states
                .insert(state.figure_id.clone(), state);
        }

        for state in import.skylanders_states {
            self.state
                .skylanders_states
                .insert(state.skylander_id.clone(), state);
        }

        self.migrate_legacy_game_state_ids();
        self.refresh_game_counts();
    }

    fn console_view(&self, console: &Console) -> ConsoleView {
        ConsoleView {
            kind: ItemKind::Console,
            id: console.id.clone(),
            name: console.name.clone(),
            manufacturer: console.manufacturer.clone(),
            year: console.year,
            variant: console.variant.clone(),
            state: self
                .state
                .console_states
                .get(&console.id)
                .map(console_state_status)
                .unwrap_or_default(),
            game_counts: self
                .game_counts_by_console
                .get(&console.id)
                .cloned()
                .unwrap_or_default(),
        }
    }

    fn game_view(&self, game: &Game, console_names: &HashMap<String, String>) -> GameView {
        GameView {
            kind: ItemKind::Game,
            id: game.id.clone(),
            title: game.title.clone(),
            year: game.year,
            publisher: game.publisher.clone(),
            console_id: game.console_id.clone(),
            console_name: console_names
                .get(&game.console_id)
                .cloned()
                .unwrap_or_else(|| game.console_id.clone()),
            state: self
                .state
                .game_states
                .get(&game.id)
                .map(game_state_status)
                .unwrap_or_default(),
        }
    }

    fn lego_view(&self, figure: &LegoDimensionFigure) -> LegoView {
        let id = figure_id(figure);
        LegoView {
            kind: ItemKind::Lego,
            id: id.clone(),
            name: figure.name.clone(),
            category: figure.category.clone(),
            year: figure.year,
            pack_id: figure.pack_id.clone(),
            state: self
                .state
                .lego_dimensions_states
                .get(&id)
                .map(lego_state_status)
                .unwrap_or_default(),
        }
    }

    fn skylander_view(&self, skylander: &Skylander) -> SkylanderView {
        let id = skylander_id(skylander);
        SkylanderView {
            kind: ItemKind::Skylander,
            id: id.clone(),
            name: skylander.name.clone(),
            game: skylander.game.clone(),
            base_color: skylander.base_color.clone(),
            category: skylander.category.clone(),
            state: self
                .state
                .skylanders_states
                .get(&id)
                .map(skylander_state_status)
                .unwrap_or_default(),
        }
    }

    fn console_names(&self) -> HashMap<String, String> {
        self.catalog
            .consoles
            .iter()
            .map(|console| (console.id.clone(), console.name.clone()))
            .collect()
    }

    fn migrate_legacy_game_state_ids(&mut self) {
        let mut migrated_states = Vec::new();

        for game in self.catalog.games.values() {
            let legacy_id = generate_legacy_id(&game.console_id, &game.title);
            if legacy_id == game.id {
                continue;
            }

            if let Some(mut legacy_state) = self.state.game_states.remove(&legacy_id) {
                legacy_state.game_id = game.id.clone();
                migrated_states.push((game.id.clone(), legacy_state));
            }
        }

        for (game_id, state) in migrated_states {
            self.state.game_states.entry(game_id).or_insert(state);
        }
    }

    fn refresh_game_counts(&mut self) {
        let mut counts: HashMap<String, ConsoleCounts> = HashMap::new();

        for state in self.state.game_states.values() {
            let console_id = get_console_from_id(&state.game_id);
            if console_id.is_empty() {
                continue;
            }

            let entry = counts.entry(console_id.to_string()).or_default();
            if state.owned {
                entry.owned += 1;
            }
            if state.favorite {
                entry.favorite += 1;
            }
            if state.wishlist {
                entry.wishlist += 1;
            }
        }

        self.game_counts_by_console = counts;
    }

    fn ensure_console(&self, id: &str) -> Result<(), CoreError> {
        self.catalog
            .consoles
            .iter()
            .any(|console| console.id == id)
            .then_some(())
            .ok_or_else(|| CoreError::UnknownItem {
                kind: ItemKind::Console,
                id: id.to_string(),
            })
    }

    fn ensure_game(&self, id: &str) -> Result<(), CoreError> {
        self.catalog
            .games
            .contains_key(id)
            .then_some(())
            .ok_or_else(|| CoreError::UnknownItem {
                kind: ItemKind::Game,
                id: id.to_string(),
            })
    }

    fn ensure_lego(&self, id: &str) -> Result<(), CoreError> {
        self.catalog
            .lego_dimensions_figures
            .iter()
            .any(|figure| figure_id(figure) == id)
            .then_some(())
            .ok_or_else(|| CoreError::UnknownItem {
                kind: ItemKind::Lego,
                id: id.to_string(),
            })
    }

    fn ensure_skylander(&self, id: &str) -> Result<(), CoreError> {
        self.catalog
            .skylanders
            .iter()
            .any(|skylander| skylander_id(skylander) == id)
            .then_some(())
            .ok_or_else(|| CoreError::UnknownItem {
                kind: ItemKind::Skylander,
                id: id.to_string(),
            })
    }
}

fn apply_status_update(
    owned: &mut bool,
    favorite: &mut bool,
    wishlist: &mut bool,
    next_owned: Option<bool>,
    next_favorite: Option<bool>,
    next_wishlist: Option<bool>,
) {
    if let Some(value) = next_owned {
        *owned = value;
    }
    if let Some(value) = next_favorite {
        *favorite = value;
    }
    if let Some(value) = next_wishlist {
        *wishlist = value;
    }
}

fn console_state_status(state: &ConsoleState) -> StatusState {
    StatusState {
        owned: state.owned,
        favorite: state.favorite,
        wishlist: state.wishlist,
        notes: state.notes.clone(),
    }
}

fn game_state_status(state: &GameState) -> StatusState {
    StatusState {
        owned: state.owned,
        favorite: state.favorite,
        wishlist: state.wishlist,
        notes: state.notes.clone(),
    }
}

fn lego_state_status(state: &LegoDimensionState) -> StatusState {
    StatusState {
        owned: state.owned,
        favorite: state.favorite,
        wishlist: state.wishlist,
        notes: state.notes.clone(),
    }
}

fn skylander_state_status(state: &SkylanderState) -> StatusState {
    StatusState {
        owned: state.owned,
        favorite: state.favorite,
        wishlist: state.wishlist,
        notes: state.notes.clone(),
    }
}
