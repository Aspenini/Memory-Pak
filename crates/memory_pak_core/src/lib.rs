use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

static DATABASE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../database");
const CONSOLES_JSON: &str = include_str!("../../../database/consoles.json");
const LEGO_DIMENSIONS_JSON: &str = include_str!("../../../database/legodimensions.json");
const SKYLANDERS_JSON: &str = include_str!("../../../database/skylanders.json");

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid import JSON: {0}")]
    InvalidImport(#[from] serde_json::Error),
    #[error("unknown item {kind:?}:{id}")]
    UnknownItem { kind: ItemKind, id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Console {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub year: u32,
    #[serde(default)]
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub year: u32,
    pub publisher: String,
    pub console_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GameState {
    pub game_id: String,
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ConsoleState {
    pub console_id: String,
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegoDimensionFigure {
    pub name: String,
    pub category: String,
    pub year: u8,
    pub pack_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LegoDimensionState {
    pub figure_id: String,
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Skylander {
    pub name: String,
    pub game: String,
    pub base_color: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SkylanderState {
    pub skylander_id: String,
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PersistedState {
    #[serde(default)]
    pub console_states: HashMap<String, ConsoleState>,
    #[serde(default)]
    pub game_states: HashMap<String, GameState>,
    #[serde(default)]
    pub lego_dimensions_states: HashMap<String, LegoDimensionState>,
    #[serde(default)]
    pub skylanders_states: HashMap<String, SkylanderState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportData {
    pub version: String,
    pub export_date: String,
    pub console_states: Vec<ConsoleState>,
    pub consoles: Vec<ConsoleExportData>,
    #[serde(default)]
    pub lego_dimensions_states: Vec<LegoDimensionState>,
    #[serde(default)]
    pub skylanders_states: Vec<SkylanderState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsoleExportData {
    pub console_id: String,
    pub games: Vec<GameState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Catalog {
    pub consoles: Vec<Console>,
    pub games: HashMap<String, Game>,
    pub lego_dimensions_figures: Vec<LegoDimensionFigure>,
    pub skylanders: Vec<Skylander>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryInput {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub filter_by: Option<String>,
    #[serde(default)]
    pub console_id: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult<T> {
    pub total: usize,
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Console,
    Game,
    Lego,
    Skylander,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusState {
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleCounts {
    pub owned: usize,
    pub favorite: usize,
    pub wishlist: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleView {
    pub kind: ItemKind,
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub year: u32,
    pub variant: Option<String>,
    pub state: StatusState,
    pub game_counts: ConsoleCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub kind: ItemKind,
    pub id: String,
    pub title: String,
    pub year: u32,
    pub publisher: String,
    pub console_id: String,
    pub console_name: String,
    pub state: StatusState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegoView {
    pub kind: ItemKind,
    pub id: String,
    pub name: String,
    pub category: String,
    pub year: u8,
    pub pack_id: String,
    pub state: StatusState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkylanderView {
    pub kind: ItemKind,
    pub id: String,
    pub name: String,
    pub game: String,
    pub base_color: String,
    pub category: String,
    pub state: StatusState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CollectionStats {
    pub total_consoles: usize,
    pub owned_consoles: usize,
    pub favorite_consoles: usize,
    pub wishlist_consoles: usize,
    pub total_games: usize,
    pub owned_games: usize,
    pub favorite_games: usize,
    pub wishlist_games: usize,
    pub total_lego_dimensions: usize,
    pub owned_lego_dimensions: usize,
    pub total_skylanders: usize,
    pub owned_skylanders: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialState {
    pub stats: CollectionStats,
    pub consoles: Vec<ConsoleView>,
    pub total_games: usize,
    pub total_lego_dimensions: usize,
    pub total_skylanders: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryPakApp {
    catalog: Catalog,
    state: PersistedState,
    game_counts_by_console: HashMap<String, ConsoleCounts>,
}

#[derive(Debug, Deserialize)]
struct DatabaseGame {
    title: String,
    publisher: String,
    #[serde(rename = "release_date")]
    release_date: Option<String>,
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

pub fn export_json_from_state(state: &PersistedState) -> Result<String, serde_json::Error> {
    let mut games_by_console: HashMap<String, Vec<GameState>> = HashMap::new();
    for (game_id, game_state) in &state.game_states {
        let console_id = get_console_from_id(game_id).to_string();
        if console_id.is_empty() {
            continue;
        }
        games_by_console
            .entry(console_id)
            .or_default()
            .push(game_state.clone());
    }

    let mut console_states: Vec<ConsoleState> = state.console_states.values().cloned().collect();
    console_states.sort_by(|a, b| a.console_id.cmp(&b.console_id));

    let mut consoles: Vec<ConsoleExportData> = games_by_console
        .into_iter()
        .map(|(console_id, mut games)| {
            games.sort_by(|a, b| a.game_id.cmp(&b.game_id));
            ConsoleExportData { console_id, games }
        })
        .collect();
    consoles.sort_by(|a, b| a.console_id.cmp(&b.console_id));

    let mut lego_dimensions_states: Vec<LegoDimensionState> =
        state.lego_dimensions_states.values().cloned().collect();
    lego_dimensions_states.sort_by(|a, b| a.figure_id.cmp(&b.figure_id));

    let mut skylanders_states: Vec<SkylanderState> =
        state.skylanders_states.values().cloned().collect();
    skylanders_states.sort_by(|a, b| a.skylander_id.cmp(&b.skylander_id));

    let export = ExportData {
        version: "1.0".to_string(),
        export_date: chrono::Utc::now().to_rfc3339(),
        console_states,
        consoles,
        lego_dimensions_states,
        skylanders_states,
    };

    serde_json::to_string_pretty(&export)
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

pub fn get_console_from_id(game_id: &str) -> &str {
    game_id
        .rsplit_once('-')
        .map(|(console, _)| console)
        .unwrap_or("")
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

pub fn generate_stable_id(
    console_id: &str,
    title: &str,
    publisher: &str,
    release_date: Option<&str>,
) -> String {
    let date = release_date.unwrap_or("");
    format!(
        "{}-{:016x}",
        console_id,
        stable_hash(&[console_id, title, publisher, date])
    )
}

pub fn generate_legacy_id(console_id: &str, title: &str) -> String {
    let mut hasher = DefaultHasher::new();
    console_id.hash(&mut hasher);
    title.hash(&mut hasher);
    format!("{}-{:x}", console_id, hasher.finish())
}

pub fn figure_id(figure: &LegoDimensionFigure) -> String {
    format!(
        "lego-{}-{}",
        slugify(&figure.pack_id),
        slugify(&figure.name)
    )
}

pub fn skylander_id(skylander: &Skylander) -> String {
    format!(
        "skylander-{}-{}-{}",
        slugify(&skylander.game),
        slugify(&skylander.name),
        slugify(&skylander.category)
    )
}

fn stable_hash(parts: &[&str]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;

    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    hash
}

fn parse_year_from_iso_date(date_str: &str) -> u32 {
    date_str
        .split('-')
        .next()
        .and_then(|year| year.parse::<u32>().ok())
        .unwrap_or(0)
}

fn slugify(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn normalize_for_search(text: &str) -> String {
    text.nfd()
        .filter(|c| {
            let code = *c as u32;
            let is_combining_mark = (0x0300..=0x036F).contains(&code)
                || (0x1AB0..=0x1AFF).contains(&code)
                || (0x1DC0..=0x1DFF).contains(&code)
                || (0x20D0..=0x20FF).contains(&code)
                || (0xFE20..=0xFE2F).contains(&code);

            !is_combining_mark
                && !matches!(*c, '\'' | ':' | '-' | '_' | '.' | ',' | '!' | '?' | ';')
        })
        .collect::<String>()
        .to_lowercase()
}

fn normalized_query(value: Option<&str>) -> Option<String> {
    let query = normalize_for_search(value.unwrap_or_default());
    (!query.trim().is_empty()).then_some(query)
}

fn normalized_filter(value: Option<&str>) -> &str {
    value.unwrap_or("all")
}

fn status_matches(state: &StatusState, filter: &str) -> bool {
    match filter {
        "owned" => state.owned,
        "favorites" | "favorite" => state.favorite,
        "wishlist" => state.wishlist,
        "notOwned" | "not_owned" => !state.owned,
        _ => true,
    }
}

fn status_score(state: &StatusState) -> u8 {
    if state.owned {
        3
    } else if state.favorite {
        2
    } else if state.wishlist {
        1
    } else {
        0
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

fn paginate<T>(items: Vec<T>, offset: Option<usize>, limit: Option<usize>) -> QueryResult<T> {
    let total = items.len();
    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(total - offset);
    let items = items.into_iter().skip(offset).take(limit).collect();

    QueryResult { total, items }
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

#[cfg(test)]
fn game_database_row_count() -> usize {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
