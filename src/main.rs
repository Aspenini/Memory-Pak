use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::*;

mod console_data;
mod game_data;
mod persistence;
mod ui;

use console_data::*;
use game_data::*;
use persistence::*;
use ui::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Console {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub year: u32,
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
    
    #[cfg(target_os = "macos")]
    {
        // Try .icns file - for now use PNG fallback since ICNS parsing is complex
        // macOS will use the PNG icon
    }
    
    // Fallback: Try to find a PNG icon
    // For Linux and as fallback, use the web icon-512.png
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
    console_states: HashMap<String, ConsoleState>,          // console_id -> ConsoleState
    games: HashMap<String, Game>,                        // game_id -> Game (flat structure)
    consoles: Vec<Console>,
    ui_state: UiState,
}

impl Default for MemoryPakApp {
    fn default() -> Self {
        Self {
            selected_console: Some("all".to_string()),
            game_states: HashMap::new(),
            console_states: HashMap::new(),
            games: HashMap::new(),
            consoles: Vec::new(),
            ui_state: UiState::default(),
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
    games_page: usize,
    consoles_page: usize,
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
}

#[derive(Default, PartialEq)]
enum SortOption {
    #[default]
    Title,
    Year,
    Status,
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

impl eframe::App for MemoryPakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Load data on first run
        if self.consoles.is_empty() {
            self.consoles = get_hardcoded_consoles();
            self.games = load_embedded_games();
            self.game_states = load_all_game_states_flat();
            self.console_states = load_all_console_states();
        }

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
                                if let Ok(import) = serde_json::from_str::<crate::ExportData>(&self.ui_state.import_text) {
                                    // Merge imported console states
                                    for console_state in import.console_states {
                                        self.console_states.insert(console_state.console_id.clone(), console_state);
                                    }
                                    
                                    // Merge imported game states (flat structure)
                                    for console_export in import.consoles {
                                        for game_state in console_export.games {
                                            self.game_states.insert(game_state.game_id.clone(), game_state);
                                        }
                                    }
                                    
                                    // Save all imported states
                                    crate::persistence::save_console_states(&self.console_states);
                                    // Group and save game states by console
                                    let mut states_by_console: HashMap<String, HashMap<String, GameState>> = HashMap::new();
                                    for (game_id, state) in &self.game_states {
                                        let console_id = game_id.split('-').next().unwrap_or("").to_string();
                                        states_by_console
                                            .entry(console_id)
                                            .or_insert_with(HashMap::new)
                                            .insert(game_id.clone(), state.clone());
                                    }
                                    for (console_id, states) in states_by_console {
                                        crate::persistence::save_game_states(&console_id, &states);
                                    }
                                    
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
                if ui.selectable_label(self.ui_state.active_tab == Tab::Consoles, "Consoles")
                    .clicked()
                {
                    self.ui_state.active_tab = Tab::Consoles;
                }
                if ui.selectable_label(self.ui_state.active_tab == Tab::Games, "Games").clicked()
                {
                    self.ui_state.active_tab = Tab::Games;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.ui_state.active_tab {
                Tab::Consoles => render_consoles_tab(ui, &self.consoles, &mut self.console_states, &self.game_states, &mut self.ui_state),
                Tab::Games => render_games_tab(
                    ui,
                    &mut self.selected_console,
                    &self.consoles,
                    &self.games,
                    &mut self.game_states,
                    &mut self.ui_state,
                ),
            }
        });
    }
}