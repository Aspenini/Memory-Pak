use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::catalog::catalog;
use crate::ids::{EntryId, EntryKind};
use crate::import_export::{apply_import, export_json_from_state, ExportData};
use crate::model::{
    Catalog, Collectible, CollectibleView, CollectionStats, CollectionView, Console, ConsoleCounts,
    ConsoleView, Game, GameView, InitialState, ItemKind, MutationResult, PersistedState,
};
use crate::query::{
    matches_filter, normalized_query, paginate, status_score, FilterBy, QueryInput, QueryResult,
    SortKey,
};

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid import JSON: {0}")]
    InvalidImport(#[from] serde_json::Error),
    #[error("unknown entry: {0}")]
    UnknownEntry(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetItemStatusInput {
    pub id: EntryId,
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
    pub id: EntryId,
    pub notes: String,
}

#[derive(Debug)]
pub struct MemoryPakApp {
    catalog: &'static Catalog,
    state: PersistedState,
    game_counts_by_console: HashMap<EntryId, ConsoleCounts>,
}

impl Default for MemoryPakApp {
    fn default() -> Self {
        Self::from_persisted_state(PersistedState::default())
    }
}

impl MemoryPakApp {
    pub fn from_persisted_state(state: PersistedState) -> Self {
        let mut app = Self {
            catalog: catalog(),
            state,
            game_counts_by_console: HashMap::new(),
        };
        app.refresh_game_counts();
        app
    }

    pub fn persisted_state(&self) -> &PersistedState {
        &self.state
    }

    pub fn catalog(&self) -> &Catalog {
        self.catalog
    }

    pub fn initial_state(&self) -> InitialState {
        let consoles = self.query_consoles(QueryInput::default()).items;
        let with_games: HashSet<EntryId> = self
            .catalog
            .games
            .values()
            .map(|g| g.console_id.clone())
            .collect();
        let consoles_with_games = consoles
            .iter()
            .filter(|v| with_games.contains(&v.id))
            .cloned()
            .collect();

        InitialState {
            stats: self.collection_stats(),
            consoles,
            consoles_with_games,
            collections: self.collection_views(),
            total_games: self.catalog.games.len(),
            total_collectibles: self.catalog.collectibles.len(),
        }
    }

    pub fn query_consoles(&self, input: QueryInput) -> QueryResult<ConsoleView> {
        let search = normalized_query(input.search.as_deref());
        let filter = input.filter_by.unwrap_or(FilterBy::All);
        let sort = input.sort_by.unwrap_or(SortKey::Name);

        let mut items: Vec<ConsoleView> = self
            .catalog
            .consoles
            .iter()
            .filter(|console| matches_console_search(console, search.as_deref()))
            .map(|console| self.console_view(console))
            .filter(|view| matches_filter(&view.state, filter))
            .collect();

        items.sort_by(|a, b| match sort {
            SortKey::Manufacturer => a
                .manufacturer
                .cmp(&b.manufacturer)
                .then_with(|| a.name.cmp(&b.name)),
            SortKey::Status => status_score(&b.state)
                .cmp(&status_score(&a.state))
                .then_with(|| a.name.cmp(&b.name)),
            _ => a.name.cmp(&b.name),
        });

        paginate(items, input.offset, input.limit)
    }

    pub fn query_games(&self, input: QueryInput) -> QueryResult<GameView> {
        let search = normalized_query(input.search.as_deref());
        let filter = input.filter_by.unwrap_or(FilterBy::All);
        let sort = input.sort_by.unwrap_or(SortKey::Title);
        let console_filter = input.console_id.as_deref();
        let console_names = self.console_names_by_id();

        let mut items: Vec<GameView> = self
            .catalog
            .games
            .values()
            .filter(|game| match console_filter {
                None | Some("all") | Some("") => true,
                Some(value) => game.console_id.as_str() == value,
            })
            .filter(|game| matches_game_search(game, search.as_deref(), &console_names))
            .map(|game| self.game_view(game, &console_names))
            .filter(|view| matches_filter(&view.state, filter))
            .collect();

        items.sort_by(|a, b| match sort {
            SortKey::Year => a.year.cmp(&b.year).then_with(|| a.title.cmp(&b.title)),
            SortKey::Status => status_score(&b.state)
                .cmp(&status_score(&a.state))
                .then_with(|| a.title.cmp(&b.title)),
            _ => a.title.cmp(&b.title),
        });

        paginate(items, input.offset, input.limit)
    }

    pub fn query_collectibles(&self, input: QueryInput) -> QueryResult<CollectibleView> {
        let search = normalized_query(input.search.as_deref());
        let filter = input.filter_by.unwrap_or(FilterBy::All);
        let sort = input.sort_by.unwrap_or(SortKey::Name);
        let collection_filter = input.collection_id.as_deref();
        let collection_names = self.collection_names_by_id();

        let mut items: Vec<CollectibleView> = self
            .catalog
            .collectibles
            .iter()
            .filter(|item| match collection_filter {
                None | Some("all") | Some("") => true,
                Some(value) => item.collection_id == value,
            })
            .filter(|item| matches_collectible_search(item, search.as_deref(), &collection_names))
            .map(|item| self.collectible_view(item, &collection_names))
            .filter(|view| matches_filter(&view.state, filter))
            .collect();

        items.sort_by(|a, b| match sort {
            SortKey::Collection => a
                .collection_name
                .cmp(&b.collection_name)
                .then_with(|| a.name.cmp(&b.name)),
            SortKey::Category => a
                .category
                .cmp(&b.category)
                .then_with(|| a.name.cmp(&b.name)),
            SortKey::Group => a.group.cmp(&b.group).then_with(|| a.name.cmp(&b.name)),
            SortKey::Variant => a.variant.cmp(&b.variant).then_with(|| a.name.cmp(&b.name)),
            SortKey::Year => a.year.cmp(&b.year).then_with(|| a.name.cmp(&b.name)),
            SortKey::Status => status_score(&b.state)
                .cmp(&status_score(&a.state))
                .then_with(|| a.name.cmp(&b.name)),
            _ => a.name.cmp(&b.name),
        });

        paginate(items, input.offset, input.limit)
    }

    pub fn set_item_status(
        &mut self,
        input: SetItemStatusInput,
    ) -> Result<MutationResult, CoreError> {
        let kind = self.ensure_entry(&input.id)?;

        let entry = self.state.entries.entry(input.id.clone()).or_default();
        if let Some(value) = input.owned {
            entry.owned = value;
        }
        if let Some(value) = input.favorite {
            entry.favorite = value;
        }
        if let Some(value) = input.wishlist {
            entry.wishlist = value;
        }
        let snapshot = entry.clone();

        if kind == EntryKind::Game {
            self.refresh_game_counts();
        }

        self.cleanup_empty(&input.id);

        Ok(MutationResult {
            id: input.id,
            state: snapshot,
            stats: self.collection_stats(),
        })
    }

    pub fn set_item_notes(
        &mut self,
        input: SetItemNotesInput,
    ) -> Result<MutationResult, CoreError> {
        self.ensure_entry(&input.id)?;

        let entry = self.state.entries.entry(input.id.clone()).or_default();
        entry.notes = input.notes;
        let snapshot = entry.clone();
        self.cleanup_empty(&input.id);

        Ok(MutationResult {
            id: input.id,
            state: snapshot,
            stats: self.collection_stats(),
        })
    }

    pub fn import_json(&mut self, json: &str) -> Result<CollectionStats, CoreError> {
        let import = serde_json::from_str::<ExportData>(json)?;
        apply_import(&mut self.state, import);
        self.state.entries.retain(|_, state| !state.is_empty());
        self.refresh_game_counts();
        Ok(self.collection_stats())
    }

    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        export_json_from_state(&self.state)
    }

    pub fn collection_stats(&self) -> CollectionStats {
        let mut stats = CollectionStats {
            total_consoles: self.catalog.consoles.len(),
            total_games: self.catalog.games.len(),
            total_collectibles: self.catalog.collectibles.len(),
            ..CollectionStats::default()
        };

        for (id, state) in &self.state.entries {
            match id.kind() {
                Some(EntryKind::Console) => {
                    if state.owned {
                        stats.owned_consoles += 1;
                    }
                    if state.favorite {
                        stats.favorite_consoles += 1;
                    }
                    if state.wishlist {
                        stats.wishlist_consoles += 1;
                    }
                }
                Some(EntryKind::Game) => {
                    if state.owned {
                        stats.owned_games += 1;
                    }
                    if state.favorite {
                        stats.favorite_games += 1;
                    }
                    if state.wishlist {
                        stats.wishlist_games += 1;
                    }
                }
                Some(EntryKind::Collectible) => {
                    if state.owned {
                        stats.owned_collectibles += 1;
                    }
                    if state.favorite {
                        stats.favorite_collectibles += 1;
                    }
                    if state.wishlist {
                        stats.wishlist_collectibles += 1;
                    }
                }
                None => {}
            }
        }

        stats
    }

    fn collection_views(&self) -> Vec<CollectionView> {
        let mut totals: HashMap<&str, (usize, usize)> = HashMap::new();
        for collectible in &self.catalog.collectibles {
            let entry = totals
                .entry(collectible.collection_id.as_str())
                .or_default();
            entry.0 += 1;
            if let Some(state) = self.state.entries.get(&collectible.id) {
                if state.owned {
                    entry.1 += 1;
                }
            }
        }
        self.catalog
            .collections
            .iter()
            .map(|c| {
                let (total, owned) = totals.get(c.id.as_str()).copied().unwrap_or((0, 0));
                CollectionView {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    manufacturer: c.manufacturer.clone(),
                    kind: c.kind.clone(),
                    total,
                    owned,
                }
            })
            .collect()
    }

    fn ensure_entry(&self, id: &EntryId) -> Result<EntryKind, CoreError> {
        let kind = id
            .kind()
            .ok_or_else(|| CoreError::UnknownEntry(id.as_str().to_string()))?;
        let exists = match kind {
            EntryKind::Console => self.catalog.consoles.iter().any(|c| &c.id == id),
            EntryKind::Game => self.catalog.games.contains_key(id),
            EntryKind::Collectible => self.catalog.collectibles.iter().any(|c| &c.id == id),
        };
        if exists {
            Ok(kind)
        } else {
            Err(CoreError::UnknownEntry(id.as_str().to_string()))
        }
    }

    fn cleanup_empty(&mut self, id: &EntryId) {
        if let Some(state) = self.state.entries.get(id) {
            if state.is_empty() {
                self.state.entries.remove(id);
            }
        }
    }

    fn refresh_game_counts(&mut self) {
        let mut counts: HashMap<EntryId, ConsoleCounts> = HashMap::new();
        for (id, state) in &self.state.entries {
            if id.kind() != Some(EntryKind::Game) {
                continue;
            }
            let Some(game) = self.catalog.games.get(id) else {
                continue;
            };
            let entry = counts.entry(game.console_id.clone()).or_default();
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

    fn console_names_by_id(&self) -> HashMap<EntryId, String> {
        self.catalog
            .consoles
            .iter()
            .map(|c| (c.id.clone(), c.name.clone()))
            .collect()
    }

    fn collection_names_by_id(&self) -> HashMap<String, String> {
        self.catalog
            .collections
            .iter()
            .map(|c| (c.id.clone(), c.name.clone()))
            .collect()
    }

    fn console_view(&self, console: &Console) -> ConsoleView {
        ConsoleView {
            kind: ItemKind::Console,
            id: console.id.clone(),
            short_id: console.short_id.clone(),
            name: console.name.clone(),
            manufacturer: console.manufacturer.clone(),
            abbreviation: console.abbreviation.clone(),
            generation: console.generation,
            state: self
                .state
                .entries
                .get(&console.id)
                .cloned()
                .unwrap_or_default(),
            game_counts: self
                .game_counts_by_console
                .get(&console.id)
                .cloned()
                .unwrap_or_default(),
        }
    }

    fn game_view(&self, game: &Game, console_names: &HashMap<EntryId, String>) -> GameView {
        GameView {
            kind: ItemKind::Game,
            id: game.id.clone(),
            title: game.title.clone(),
            year: game.year,
            developer: game.developer.clone(),
            publisher: game.publisher.clone(),
            console_id: game.console_id.clone(),
            console_name: console_names
                .get(&game.console_id)
                .cloned()
                .unwrap_or_else(|| game.console_short_id.clone()),
            state: self
                .state
                .entries
                .get(&game.id)
                .cloned()
                .unwrap_or_default(),
        }
    }

    fn collectible_view(
        &self,
        item: &Collectible,
        collection_names: &HashMap<String, String>,
    ) -> CollectibleView {
        CollectibleView {
            kind: ItemKind::Collectible,
            id: item.id.clone(),
            collection_id: item.collection_id.clone(),
            collection_name: collection_names
                .get(&item.collection_id)
                .cloned()
                .unwrap_or_else(|| item.collection_id.clone()),
            name: item.name.clone(),
            category: item.category.clone(),
            group: item.group.clone(),
            variant: item.variant.clone(),
            year: item.year,
            state: self
                .state
                .entries
                .get(&item.id)
                .cloned()
                .unwrap_or_default(),
        }
    }
}

fn matches_console_search(console: &Console, query: Option<&str>) -> bool {
    let Some(q) = query else { return true };
    crate::ids::normalize_for_search(&console.name).contains(q)
        || crate::ids::normalize_for_search(&console.manufacturer).contains(q)
        || crate::ids::normalize_for_search(&console.abbreviation).contains(q)
}

fn matches_game_search(
    game: &Game,
    query: Option<&str>,
    console_names: &HashMap<EntryId, String>,
) -> bool {
    let Some(q) = query else { return true };
    crate::ids::normalize_for_search(&game.title).contains(q)
        || crate::ids::normalize_for_search(&game.publisher).contains(q)
        || crate::ids::normalize_for_search(&game.developer).contains(q)
        || console_names
            .get(&game.console_id)
            .map(|name| crate::ids::normalize_for_search(name).contains(q))
            .unwrap_or(false)
}

fn matches_collectible_search(
    item: &Collectible,
    query: Option<&str>,
    collection_names: &HashMap<String, String>,
) -> bool {
    let Some(q) = query else { return true };
    crate::ids::normalize_for_search(&item.name).contains(q)
        || crate::ids::normalize_for_search(&item.category).contains(q)
        || crate::ids::normalize_for_search(&item.group).contains(q)
        || crate::ids::normalize_for_search(&item.variant).contains(q)
        || collection_names
            .get(&item.collection_id)
            .map(|name| crate::ids::normalize_for_search(name).contains(q))
            .unwrap_or(false)
}
