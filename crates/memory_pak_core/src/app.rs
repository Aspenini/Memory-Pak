use std::collections::{HashMap, HashSet};

use memory_pak_catalog::{query_grams, CatalogArchive, KindIndex, SearchIndex};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::catalog::{archive, catalog};
use crate::ids::{EntryId, EntryKind};
use crate::import_export::{
    apply_import, decode_persisted_state, encode_persisted_state, export_json_from_state,
    ExportData, SaveDecodeError,
};
use crate::model::{
    Catalog, CatalogRef, Collectible, CollectibleView, CollectionStats, CollectionView, Console,
    ConsoleCounts, ConsoleView, Game, GameView, InitialState, ItemKind, MutationResult,
    PersistedState, QuerySnapshot, RowView,
};
use crate::query::{normalized_query, FilterBy, QueryInput, QueryResult, QuerySpec, SortKey};

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid import JSON: {0}")]
    InvalidImport(#[from] serde_json::Error),
    #[error("unknown entry: {0}")]
    UnknownEntry(String),
    #[error(transparent)]
    InvalidSave(#[from] SaveDecodeError),
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
    state_index: StateIndex,
    game_counts_by_console: HashMap<EntryId, ConsoleCounts>,
}

#[derive(Debug, Default)]
struct KindStateBits {
    owned: Vec<bool>,
    favorite: Vec<bool>,
    wishlist: Vec<bool>,
}

#[derive(Debug, Default)]
struct StateIndex {
    consoles: KindStateBits,
    games: KindStateBits,
    collectibles: KindStateBits,
}

impl Default for MemoryPakApp {
    fn default() -> Self {
        Self::from_persisted_state(PersistedState::default())
    }
}

impl MemoryPakApp {
    pub fn from_persisted_state(state: PersistedState) -> Self {
        let mut state = state;
        state.entries.retain(|_, entry| !entry.is_empty());
        let mut app = Self {
            catalog: catalog(),
            state,
            state_index: StateIndex::default(),
            game_counts_by_console: HashMap::new(),
        };
        app.rebuild_state_index();
        app.refresh_game_counts();
        app
    }

    pub fn from_save_json(json: &str) -> Result<Self, CoreError> {
        Ok(Self::from_persisted_state(decode_persisted_state(json)?))
    }

    pub fn persisted_state(&self) -> &PersistedState {
        &self.state
    }

    pub fn save_json(&self) -> Result<String, serde_json::Error> {
        encode_persisted_state(&self.state)
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
        let snapshot = self.query_snapshot(QuerySpec {
            kind: ItemKind::Console,
            search: input.search,
            sort_by: input.sort_by,
            filter_by: input.filter_by,
            offset: input.offset,
            limit: input.limit,
            ..Default::default()
        });
        QueryResult {
            total: snapshot.total,
            items: snapshot
                .items
                .into_iter()
                .filter_map(|reference| match self.row_view(reference) {
                    Some(RowView::Console(view)) => Some(view),
                    _ => None,
                })
                .collect(),
        }
    }

    pub fn query_games(&self, input: QueryInput) -> QueryResult<GameView> {
        let snapshot = self.query_snapshot(QuerySpec {
            kind: ItemKind::Game,
            search: input.search,
            sort_by: input.sort_by,
            filter_by: input.filter_by,
            group_id: input.console_id,
            offset: input.offset,
            limit: input.limit,
        });
        QueryResult {
            total: snapshot.total,
            items: snapshot
                .items
                .into_iter()
                .filter_map(|reference| match self.row_view(reference) {
                    Some(RowView::Game(view)) => Some(view),
                    _ => None,
                })
                .collect(),
        }
    }

    pub fn query_collectibles(&self, input: QueryInput) -> QueryResult<CollectibleView> {
        let snapshot = self.query_snapshot(QuerySpec {
            kind: ItemKind::Collectible,
            search: input.search,
            sort_by: input.sort_by,
            filter_by: input.filter_by,
            group_id: input.collection_id,
            offset: input.offset,
            limit: input.limit,
        });
        QueryResult {
            total: snapshot.total,
            items: snapshot
                .items
                .into_iter()
                .filter_map(|reference| match self.row_view(reference) {
                    Some(RowView::Collectible(view)) => Some(view),
                    _ => None,
                })
                .collect(),
        }
    }

    pub fn query_snapshot(&self, spec: QuerySpec) -> QuerySnapshot {
        let catalog = archive(self.catalog);
        let (len, index) = match spec.kind {
            ItemKind::Console => (catalog.consoles.len(), &catalog.console_index),
            ItemKind::Game => (catalog.games.len(), &catalog.game_index),
            ItemKind::Collectible => (catalog.collectibles.len(), &catalog.collectible_index),
        };
        let normalized = normalized_query(spec.search.as_deref());
        let mut candidates = match normalized.as_deref() {
            Some(query) => search_candidates(catalog, spec.kind, query),
            None => (0..len as u32).collect(),
        };

        if let Some(group) = spec
            .group_id
            .as_deref()
            .filter(|value| !value.is_empty() && *value != "all")
        {
            if let Some(facet) = index
                .facets
                .iter()
                .find(|facet| catalog.string(facet.value) == group)
            {
                candidates = intersect_sorted(&candidates, &facet.ordinals);
            } else {
                candidates.clear();
            }
        }

        let filter = spec.filter_by.unwrap_or(FilterBy::All);
        if filter != FilterBy::All {
            candidates.retain(|ordinal| {
                let reference = CatalogRef {
                    kind: spec.kind,
                    ordinal: *ordinal,
                };
                self.matches_ref_filter(reference, filter)
            });
        }

        let candidate_count = candidates.len();
        let mut candidate_bits = vec![false; len];
        for ordinal in candidates {
            candidate_bits[ordinal as usize] = true;
        }
        let default_sort = match spec.kind {
            ItemKind::Game => SortKey::Title,
            _ => SortKey::Name,
        };
        let sort = spec.sort_by.unwrap_or(default_sort);
        let base_order = order_for(index, sort, len);
        let mut ordered: Vec<u32> = if sort == SortKey::Status {
            let mut buckets: [Vec<u32>; 4] = Default::default();
            for ordinal in base_order {
                if candidate_bits[ordinal as usize] {
                    let reference = CatalogRef {
                        kind: spec.kind,
                        ordinal,
                    };
                    buckets[usize::from(self.status_score_ref(reference))].push(ordinal);
                }
            }
            buckets.into_iter().rev().flatten().collect()
        } else if candidate_count < len / 2 {
            let ranks = ranks_for(index, sort);
            let mut matches: Vec<u32> = candidate_bits
                .iter()
                .enumerate()
                .filter_map(|(ordinal, included)| included.then_some(ordinal as u32))
                .collect();
            if ranks.len() == len {
                matches.sort_by_key(|ordinal| ranks[*ordinal as usize]);
            }
            matches
        } else {
            base_order
                .into_iter()
                .filter(|ordinal| candidate_bits[*ordinal as usize])
                .collect()
        };

        let total = ordered.len();
        let offset = spec.offset.unwrap_or(0).min(total);
        let limit = spec.limit.unwrap_or(total - offset);
        ordered = ordered.into_iter().skip(offset).take(limit).collect();
        QuerySnapshot {
            total,
            items: ordered
                .into_iter()
                .map(|ordinal| CatalogRef {
                    kind: spec.kind,
                    ordinal,
                })
                .collect(),
        }
    }

    pub fn row_view(&self, reference: CatalogRef) -> Option<RowView> {
        match reference.kind {
            ItemKind::Console => self
                .catalog
                .consoles
                .get(reference.ordinal as usize)
                .map(|console| RowView::Console(self.console_view(console))),
            ItemKind::Game => {
                let record = archive(self.catalog)
                    .games
                    .get(reference.ordinal as usize)?;
                let id = archive(self.catalog).string(record.id);
                let game = self.catalog.games.get(id)?;
                Some(RowView::Game(self.game_view(game)))
            }
            ItemKind::Collectible => self
                .catalog
                .collectibles
                .get(reference.ordinal as usize)
                .map(|item| RowView::Collectible(self.collectible_view(item))),
        }
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
        self.rebuild_state_index();

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
        self.rebuild_state_index();

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
        self.rebuild_state_index();
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
            match self.known_kind(id.as_str()) {
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
        self.known_kind(id.as_str())
            .ok_or_else(|| CoreError::UnknownEntry(id.as_str().to_string()))
    }

    fn known_kind(&self, id: &str) -> Option<EntryKind> {
        let catalog = archive(self.catalog);
        match EntryId::from_raw(id.to_string()).kind()? {
            EntryKind::Console => catalog
                .consoles
                .binary_search_by(|record| catalog.string(record.id).cmp(id))
                .is_ok()
                .then_some(EntryKind::Console),
            EntryKind::Game => catalog
                .games
                .binary_search_by(|record| catalog.string(record.id).cmp(id))
                .is_ok()
                .then_some(EntryKind::Game),
            EntryKind::Collectible => catalog
                .collectibles
                .binary_search_by(|record| catalog.string(record.id).cmp(id))
                .is_ok()
                .then_some(EntryKind::Collectible),
        }
    }

    fn cleanup_empty(&mut self, id: &EntryId) {
        if let Some(state) = self.state.entries.get(id) {
            if state.is_empty() {
                self.state.entries.remove(id);
            }
        }
    }

    fn rebuild_state_index(&mut self) {
        let archive = archive(self.catalog);
        self.state_index = StateIndex {
            consoles: KindStateBits::new(archive.consoles.len()),
            games: KindStateBits::new(archive.games.len()),
            collectibles: KindStateBits::new(archive.collectibles.len()),
        };
        for (id, state) in &self.state.entries {
            let reference = match id.kind() {
                Some(EntryKind::Console) => archive
                    .consoles
                    .binary_search_by(|record| archive.string(record.id).cmp(id.as_str()))
                    .ok()
                    .map(|ordinal| (ItemKind::Console, ordinal)),
                Some(EntryKind::Game) => archive
                    .games
                    .binary_search_by(|record| archive.string(record.id).cmp(id.as_str()))
                    .ok()
                    .map(|ordinal| (ItemKind::Game, ordinal)),
                Some(EntryKind::Collectible) => archive
                    .collectibles
                    .binary_search_by(|record| archive.string(record.id).cmp(id.as_str()))
                    .ok()
                    .map(|ordinal| (ItemKind::Collectible, ordinal)),
                None => continue,
            };
            let Some((kind, ordinal)) = reference else {
                continue;
            };
            let bits = match kind {
                ItemKind::Console => &mut self.state_index.consoles,
                ItemKind::Game => &mut self.state_index.games,
                ItemKind::Collectible => &mut self.state_index.collectibles,
            };
            bits.owned[ordinal] = state.owned;
            bits.favorite[ordinal] = state.favorite;
            bits.wishlist[ordinal] = state.wishlist;
        }
    }

    fn state_bits(&self, reference: CatalogRef) -> &KindStateBits {
        match reference.kind {
            ItemKind::Console => &self.state_index.consoles,
            ItemKind::Game => &self.state_index.games,
            ItemKind::Collectible => &self.state_index.collectibles,
        }
    }

    fn matches_ref_filter(&self, reference: CatalogRef, filter: FilterBy) -> bool {
        let ordinal = reference.ordinal as usize;
        let bits = self.state_bits(reference);
        match filter {
            FilterBy::All => true,
            FilterBy::Owned => bits.owned[ordinal],
            FilterBy::Favorites => bits.favorite[ordinal],
            FilterBy::Wishlist => bits.wishlist[ordinal],
            FilterBy::NotOwned => !bits.owned[ordinal],
        }
    }

    fn status_score_ref(&self, reference: CatalogRef) -> u8 {
        let ordinal = reference.ordinal as usize;
        let bits = self.state_bits(reference);
        if bits.owned[ordinal] {
            3
        } else if bits.favorite[ordinal] {
            2
        } else if bits.wishlist[ordinal] {
            1
        } else {
            0
        }
    }

    fn refresh_game_counts(&mut self) {
        let mut counts: HashMap<EntryId, ConsoleCounts> = HashMap::new();
        for game in self.catalog.games.values() {
            counts.entry(game.console_id.clone()).or_default().total += 1;
        }
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

    fn game_view(&self, game: &Game) -> GameView {
        GameView {
            kind: ItemKind::Game,
            id: game.id.clone(),
            title: game.title.clone(),
            year: game.year,
            developer: game.developer.clone(),
            publisher: game.publisher.clone(),
            console_id: game.console_id.clone(),
            console_name: self
                .catalog
                .consoles
                .binary_search_by(|console| console.id.as_str().cmp(game.console_id.as_str()))
                .ok()
                .map(|index| self.catalog.consoles[index].name.clone())
                .unwrap_or_else(|| game.console_short_id.clone()),
            state: self
                .state
                .entries
                .get(&game.id)
                .cloned()
                .unwrap_or_default(),
        }
    }

    fn collectible_view(&self, item: &Collectible) -> CollectibleView {
        CollectibleView {
            kind: ItemKind::Collectible,
            id: item.id.clone(),
            collection_id: item.collection_id.clone(),
            collection_name: self
                .catalog
                .collections
                .binary_search_by(|collection| collection.id.cmp(&item.collection_id))
                .ok()
                .map(|index| self.catalog.collections[index].name.clone())
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

impl KindStateBits {
    fn new(len: usize) -> Self {
        Self {
            owned: vec![false; len],
            favorite: vec![false; len],
            wishlist: vec![false; len],
        }
    }
}

fn search_candidates(catalog: &CatalogArchive, kind: ItemKind, query: &str) -> Vec<u32> {
    let index = match kind {
        ItemKind::Console => &catalog.console_index.search,
        ItemKind::Game => &catalog.game_index.search,
        ItemKind::Collectible => &catalog.collectible_index.search,
    };
    let mut postings = query_grams(query)
        .into_iter()
        .map(|gram| posting_for(catalog, index, &gram))
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default();
    if postings.is_empty() {
        return Vec::new();
    }
    postings.sort_by_key(Vec::len);
    let mut candidates = postings.remove(0);
    for values in postings {
        candidates = intersect_sorted(&candidates, &values);
        if candidates.is_empty() {
            break;
        }
    }
    candidates.retain(|ordinal| {
        let search_text = match kind {
            ItemKind::Console => catalog
                .consoles
                .get(*ordinal as usize)
                .map(|record| catalog.string(record.search_text)),
            ItemKind::Game => catalog
                .games
                .get(*ordinal as usize)
                .map(|record| catalog.string(record.search_text)),
            ItemKind::Collectible => catalog
                .collectibles
                .get(*ordinal as usize)
                .map(|record| catalog.string(record.search_text)),
        };
        search_text.is_some_and(|value| value.contains(query))
    });
    candidates
}

fn posting_for(catalog: &CatalogArchive, index: &SearchIndex, gram: &str) -> Option<Vec<u32>> {
    let position = index
        .terms
        .binary_search_by(|entry| catalog.string(entry.term).cmp(gram))
        .ok()?;
    let entry = &index.terms[position];
    let start = entry.start as usize;
    let end = start + entry.len as usize;
    let deltas = index.postings.get(start..end)?;
    let mut ordinal = 0;
    Some(
        deltas
            .iter()
            .enumerate()
            .map(|(index, delta)| {
                if index == 0 {
                    ordinal = *delta;
                } else {
                    ordinal += *delta;
                }
                ordinal
            })
            .collect(),
    )
}

fn intersect_sorted(left: &[u32], right: &[u32]) -> Vec<u32> {
    let mut output = Vec::with_capacity(left.len().min(right.len()));
    let (mut a, mut b) = (0, 0);
    while a < left.len() && b < right.len() {
        match left[a].cmp(&right[b]) {
            std::cmp::Ordering::Less => a += 1,
            std::cmp::Ordering::Greater => b += 1,
            std::cmp::Ordering::Equal => {
                output.push(left[a]);
                a += 1;
                b += 1;
            }
        }
    }
    output
}

fn order_for(index: &KindIndex, sort: SortKey, len: usize) -> Vec<u32> {
    let values = match sort {
        SortKey::Manufacturer => &index.orders.manufacturer,
        SortKey::Year => &index.orders.year,
        SortKey::Collection => &index.orders.collection,
        SortKey::Category => &index.orders.category,
        SortKey::Group => &index.orders.group,
        SortKey::Variant => &index.orders.variant,
        SortKey::Status | SortKey::Title | SortKey::Name => &index.orders.name,
    };
    if values.is_empty() {
        (0..len as u32).collect()
    } else {
        values.clone()
    }
}

fn ranks_for(index: &KindIndex, sort: SortKey) -> &[u32] {
    match sort {
        SortKey::Manufacturer => &index.ranks.manufacturer,
        SortKey::Year => &index.ranks.year,
        SortKey::Collection => &index.ranks.collection,
        SortKey::Category => &index.ranks.category,
        SortKey::Group => &index.ranks.group,
        SortKey::Variant => &index.ranks.variant,
        SortKey::Status | SortKey::Title | SortKey::Name => &index.ranks.name,
    }
}
