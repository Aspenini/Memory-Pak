use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct Catalog {
    pub consoles: Vec<Console>,
    pub games: HashMap<String, Game>,
    pub lego_dimensions_figures: Vec<LegoDimensionFigure>,
    pub skylanders: Vec<Skylander>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Console,
    Game,
    Lego,
    Skylander,
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
