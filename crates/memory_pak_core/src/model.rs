use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ids::{EntryId, EntryKind};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Console {
    pub id: EntryId,
    pub short_id: String,
    pub name: String,
    pub manufacturer: String,
    pub family: String,
    pub form_factor: String,
    pub generation: u32,
    pub abbreviation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: EntryId,
    pub console_id: EntryId,
    pub console_short_id: String,
    pub title: String,
    pub developer: String,
    pub publisher: String,
    pub year: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Collectible {
    pub id: EntryId,
    pub collection_id: String,
    pub name: String,
    pub category: String,
    pub group: String,
    pub variant: String,
    pub year: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntryState {
    #[serde(default)]
    pub owned: bool,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub wishlist: bool,
    #[serde(default)]
    pub notes: String,
}

impl EntryState {
    pub fn is_empty(&self) -> bool {
        !self.owned && !self.favorite && !self.wishlist && self.notes.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PersistedState {
    #[serde(default)]
    pub entries: HashMap<EntryId, EntryState>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub consoles: Vec<Console>,
    pub games: HashMap<EntryId, Game>,
    pub collections: Vec<Collection>,
    pub collectibles: Vec<Collectible>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Console,
    Game,
    Collectible,
}

impl From<EntryKind> for ItemKind {
    fn from(value: EntryKind) -> Self {
        match value {
            EntryKind::Console => ItemKind::Console,
            EntryKind::Game => ItemKind::Game,
            EntryKind::Collectible => ItemKind::Collectible,
        }
    }
}

impl From<ItemKind> for EntryKind {
    fn from(value: ItemKind) -> Self {
        match value {
            ItemKind::Console => EntryKind::Console,
            ItemKind::Game => EntryKind::Game,
            ItemKind::Collectible => EntryKind::Collectible,
        }
    }
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
    pub id: EntryId,
    pub short_id: String,
    pub name: String,
    pub manufacturer: String,
    pub abbreviation: String,
    pub generation: u32,
    pub state: EntryState,
    pub game_counts: ConsoleCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub kind: ItemKind,
    pub id: EntryId,
    pub title: String,
    pub year: u32,
    pub developer: String,
    pub publisher: String,
    pub console_id: EntryId,
    pub console_name: String,
    pub state: EntryState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectibleView {
    pub kind: ItemKind,
    pub id: EntryId,
    pub collection_id: String,
    pub collection_name: String,
    pub name: String,
    pub category: String,
    pub group: String,
    pub variant: String,
    pub year: u8,
    pub state: EntryState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionView {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub kind: String,
    pub total: usize,
    pub owned: usize,
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
    pub total_collectibles: usize,
    pub owned_collectibles: usize,
    pub favorite_collectibles: usize,
    pub wishlist_collectibles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialState {
    pub stats: CollectionStats,
    pub consoles: Vec<ConsoleView>,
    /// Consoles that have at least one game in the catalog (for the Games tab filter only).
    pub consoles_with_games: Vec<ConsoleView>,
    pub collections: Vec<CollectionView>,
    pub total_games: usize,
    pub total_collectibles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationResult {
    pub id: EntryId,
    pub state: EntryState,
    pub stats: CollectionStats,
}
