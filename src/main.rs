#![windows_subsystem = "windows"]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::*;

mod console_data;
mod game_data;
mod lego_dimensions;
mod skylanders;
mod persistence;
mod ui;

use console_data::*;
use game_data::*;
use lego_dimensions::*;
use skylanders::*;
use persistence::*;
use ui::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Console {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub year: u32,
    #[serde(default)]
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub year: u32,
    pub publisher: String,
    pub console_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameState {
    pub game_id: String,
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsoleState {
    pub console_id: String,
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleExportData {
    pub console_id: String,
    pub games: Vec<GameState>,
}

fn main() -> Result<(), eframe::Error> {
    // Load app icon
    let icon = load_app_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_title("Memory Pak");

    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
    }

    eframe::run_native(
        "Memory Pak",
        options,
        Box::new(|_cc| Box::new(MemoryPakApp::default())),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn load_app_icon() -> Option<egui::IconData> {
    // Try platform-specific icons first
    #[cfg(target_os = "windows")]
    {
        let icon_bytes = include_bytes!("../icons/windows/AppIcon.ico");
        if let Ok(img) = image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Ico) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            return Some(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            });
        }
    }

    // Fallback: Use PNG icon for macOS, Linux, and Windows if ICO fails
    {
        let icon_bytes = include_bytes!("../icons/web/icon-512.png");
        if let Ok(img) = image::load_from_memory(icon_bytes) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            return Some(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            });
        }
    }

    None
}

#[cfg(target_arch = "wasm32")]
fn load_app_icon() -> Option<egui::IconData> {
    // No icon needed for web
    None
}

struct MemoryPakApp {
    selected_console: Option<String>, // "all" means all consoles, otherwise console_id
    game_states: HashMap<String, GameState>, // game_id -> GameState (flat structure)
    console_states: HashMap<String, ConsoleState>, // console_id -> ConsoleState
    games: HashMap<String, Game>,     // game_id -> Game (flat structure)
    consoles: Vec<Console>,
    lego_dimensions_figures: Vec<LegoDimensionFigure>,
    lego_dimensions_states: HashMap<String, LegoDimensionState>,
    skylanders: Vec<Skylander>,
    skylanders_states: HashMap<String, SkylanderState>,
    ui_state: UiState,
    data_loaded: bool,
    game_counts_by_console: HashMap<String, (usize, usize, usize)>, // (owned, favorite, wishlist)
    pending_console_save: bool,
    pending_game_save: bool,
    pending_lego_save: bool,
    pending_skylanders_save: bool,
    last_save_time: Option<std::time::Instant>,
}

impl Default for MemoryPakApp {
    fn default() -> Self {
        Self {
            selected_console: Some("all".to_string()),
            game_states: HashMap::new(),
            console_states: HashMap::new(),
            games: HashMap::new(),
            consoles: Vec::new(),
            lego_dimensions_figures: Vec::new(),
            lego_dimensions_states: HashMap::new(),
            skylanders: Vec::new(),
            skylanders_states: HashMap::new(),
            ui_state: UiState::default(),
            data_loaded: false,
            game_counts_by_console: HashMap::new(),
            pending_console_save: false,
            pending_game_save: false,
            pending_lego_save: false,
            pending_skylanders_save: false,
            last_save_time: None,
        }
    }
}

#[derive(Default)]
struct UiState {
    active_tab: Tab,
    sort_by: SortOption,
    console_sort_by: SortOption,
    filter_by: FilterOption,
    console_filter_by: FilterOption,
    search_query: String,
    console_search_query: String,
    games_page: usize,
    consoles_page: usize,
    lego_sort_by: LegoSortOption,
    lego_filter_by: FilterOption,
    lego_search_query: String,
    skylanders_sort_by: SkylandersSortOption,
    skylanders_filter_by: FilterOption,
    skylanders_search_query: String,
    // Cached lowercase strings for search
    search_query_lower: String,
    console_search_query_lower: String,
    lego_search_query_lower: String,
    skylanders_search_query_lower: String,
    // Track last search/filter to detect changes
    last_console_search: String,
    last_console_filter: String,
    #[cfg(target_arch = "wasm32")]
    needs_import: bool,
    #[cfg(target_arch = "wasm32")]
    import_text: String,
}

#[derive(Default, PartialEq)]
enum Tab {
    #[default]
    Consoles,
    Games,
    LegoDimensions,
    Skylanders,
}

#[derive(Default, PartialEq)]
enum SortOption {
    #[default]
    Title,
    Year,
    Status,
}

#[derive(Default, PartialEq)]
enum LegoSortOption {
    #[default]
    Name,
    Category,
    Year,
    Pack,
}

#[derive(Default, PartialEq)]
pub enum SkylandersSortOption {
    #[default]
    Name,
    Game,
    BaseColor,
    Category,
}

#[derive(Default, PartialEq)]
enum FilterOption {
    #[default]
    All,
    Owned,
    Favorites,
    Wishlist,
    NotOwned,
}

impl MemoryPakApp {
    /// Compute game counts by console from game_states
    fn compute_game_counts(game_states: &HashMap<String, GameState>) -> HashMap<String, (usize, usize, usize)> {
        use crate::game_data::get_console_from_id;
        let mut counts: HashMap<String, (usize, usize, usize)> = HashMap::new();
        
        for state in game_states.values() {
            let console_id = get_console_from_id(&state.game_id);
            let entry = counts.entry(console_id.to_string()).or_insert((0, 0, 0));
            if state.owned { entry.0 += 1; }
            if state.favorite { entry.1 += 1; }
            if state.wishlist { entry.2 += 1; }
        }
        
        counts
    }

    /// Invalidate and recompute game counts cache
    pub fn invalidate_game_counts_cache(&mut self) {
        self.game_counts_by_console = Self::compute_game_counts(&self.game_states);
    }

    /// Flush pending saves if enough time has passed (500ms debounce)
    fn maybe_flush_saves(&mut self) {
        const SAVE_DEBOUNCE_MS: u64 = 500;
        
        // Check if we have any pending saves
        if !self.pending_console_save && !self.pending_game_save && !self.pending_lego_save && !self.pending_skylanders_save {
            return; // Nothing to save
        }

        // Check if enough time has passed since last save
        if let Some(last_save) = self.last_save_time {
            if last_save.elapsed().as_millis() < SAVE_DEBOUNCE_MS as u128 {
                return; // Not enough time has passed
            }
        }

        // Flush all pending saves
        if self.pending_console_save {
            crate::persistence::save_console_states(&self.console_states);
            self.pending_console_save = false;
        }

        if self.pending_game_save {
            // Group and save game states by console
            let mut states_by_console: HashMap<String, HashMap<String, GameState>> = HashMap::new();
            for (game_id, state) in &self.game_states {
                let console_id = crate::game_data::get_console_from_id(game_id);
                states_by_console
                    .entry(console_id.to_string())
                    .or_insert_with(HashMap::new)
                    .insert(game_id.clone(), state.clone());
            }
            for (console_id, console_states) in states_by_console {
                crate::persistence::save_game_states(&console_id, &console_states);
            }
            // Invalidate cache after saving game states
            self.invalidate_game_counts_cache();
            self.pending_game_save = false;
        }

        if self.pending_lego_save {
            crate::persistence::save_lego_dimensions_states(&self.lego_dimensions_states);
            self.pending_lego_save = false;
        }

        if self.pending_skylanders_save {
            crate::persistence::save_skylanders_states(&self.skylanders_states);
            self.pending_skylanders_save = false;
        }

        // Update last save time
        self.last_save_time = Some(std::time::Instant::now());
    }
}

impl eframe::App for MemoryPakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Load data on first run
        if !self.data_loaded {
            self.consoles = get_hardcoded_consoles();
            self.games = load_embedded_games();
            self.game_states = load_all_game_states_flat();
            self.console_states = load_all_console_states();
            self.lego_dimensions_figures = load_lego_dimensions_figures();
            self.lego_dimensions_states = load_lego_dimensions_states();
            self.skylanders = load_skylanders();
            self.skylanders_states = load_skylanders_states();
            // Compute initial game counts
            self.game_counts_by_console = Self::compute_game_counts(&self.game_states);
            self.data_loaded = true;
        }

        // Invalidate cache immediately when game states first change in a save cycle
        // This ensures the cache is up-to-date even before the save debounce completes
        // We only do this once per save cycle (when last_save_time is None)
        if self.pending_game_save && self.last_save_time.is_none() {
            self.invalidate_game_counts_cache();
        }

        // Flush pending saves if debounce time has passed
        self.maybe_flush_saves();

        // Handle web import dialog
        #[cfg(target_arch = "wasm32")]
        {
            if self.ui_state.needs_import {
                egui::Window::new("Import Data")
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.label("Paste your exported JSON data:");
                        ui.text_edit_multiline(&mut self.ui_state.import_text);
                        ui.horizontal(|ui| {
                            if ui.button("Import").clicked() {
                                if let Ok(import) = serde_json::from_str::<crate::ExportData>(
                                    &self.ui_state.import_text,
                                ) {
                                    // Merge imported console states
                                    for console_state in import.console_states {
                                        self.console_states.insert(
                                            console_state.console_id.clone(),
                                            console_state,
                                        );
                                    }

                                    // Merge imported game states (flat structure)
                                    for console_export in import.consoles {
                                        for game_state in console_export.games {
                                            self.game_states
                                                .insert(game_state.game_id.clone(), game_state);
                                        }
                                    }
                                    // Invalidate cache after import
                                    self.invalidate_game_counts_cache();

                                    // Merge imported LEGO Dimensions states
                                    for figure_state in import.lego_dimensions_states {
                                        self.lego_dimensions_states
                                            .insert(figure_state.figure_id.clone(), figure_state);
                                    }

                                    // Merge imported Skylanders states
                                    for skylander_state in import.skylanders_states {
                                        self.skylanders_states
                                            .insert(skylander_state.skylander_id.clone(), skylander_state);
                                    }

                                    // Save all imported states
                                    crate::persistence::save_console_states(&self.console_states);
                                    // Group and save game states by console
                                    let mut states_by_console: HashMap<
                                        String,
                                        HashMap<String, GameState>,
                                    > = HashMap::new();
                                    for (game_id, state) in &self.game_states {
                                        let console_id =
                                            game_id.split('-').next().unwrap_or("").to_string();
                                        states_by_console
                                            .entry(console_id)
                                            .or_insert_with(HashMap::new)
                                            .insert(game_id.clone(), state.clone());
                                    }
                                    for (console_id, states) in states_by_console {
                                        crate::persistence::save_game_states(&console_id, &states);
                                    }
                                    crate::persistence::save_lego_dimensions_states(
                                        &self.lego_dimensions_states,
                                    );
                                    crate::persistence::save_skylanders_states(
                                        &self.skylanders_states,
                                    );

                                    self.ui_state.needs_import = false;
                                    self.ui_state.import_text.clear();
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                self.ui_state.needs_import = false;
                                self.ui_state.import_text.clear();
                            }
                        });
                    });
            }
        }

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Memory Pak");
                ui.separator();
                if ui.button("Import").clicked() {
                    if let Err(e) = import_data(self) {
                        eprintln!("Import error: {:?}", e);
                    }
                }
                if ui.button("Export").clicked() {
                    if let Err(e) = export_data(self) {
                        eprintln!("Export error: {:?}", e);
                    }
                }
            });
        });

        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Tabs");
                ui.separator();
                if ui
                    .selectable_label(self.ui_state.active_tab == Tab::Consoles, "Consoles")
                    .clicked()
                {
                    self.ui_state.active_tab = Tab::Consoles;
                }
                if ui
                    .selectable_label(self.ui_state.active_tab == Tab::Games, "Games")
                    .clicked()
                {
                    self.ui_state.active_tab = Tab::Games;
                }
                if ui
                    .selectable_label(
                        self.ui_state.active_tab == Tab::LegoDimensions,
                        "LEGO Dimensions",
                    )
                    .clicked()
                {
                    self.ui_state.active_tab = Tab::LegoDimensions;
                }
                if ui
                    .selectable_label(
                        self.ui_state.active_tab == Tab::Skylanders,
                        "Skylanders",
                    )
                    .clicked()
                {
                    self.ui_state.active_tab = Tab::Skylanders;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.ui_state.active_tab {
            Tab::Consoles => render_consoles_tab(
                ui,
                &self.consoles,
                &mut self.console_states,
                &self.game_counts_by_console,
                &mut self.ui_state,
                &mut self.pending_console_save,
            ),
            Tab::Games => render_games_tab(
                ui,
                &mut self.selected_console,
                &self.consoles,
                &self.games,
                &mut self.game_states,
                &mut self.ui_state,
                &mut self.pending_game_save,
            ),
            Tab::LegoDimensions => render_lego_dimensions_tab(
                ui,
                &self.lego_dimensions_figures,
                &mut self.lego_dimensions_states,
                &mut self.ui_state,
                &mut self.pending_lego_save,
            ),
            Tab::Skylanders => render_skylanders_tab(
                ui,
                &self.skylanders,
                &mut self.skylanders_states,
                &mut self.ui_state,
                &mut self.pending_skylanders_save,
            ),
        });
    }
}
