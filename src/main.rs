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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Memory Pak"),
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

#[derive(Default)]
struct MemoryPakApp {
    selected_console: Option<String>,
    game_states: HashMap<String, HashMap<String, GameState>>, // console_id -> game_id -> GameState
    console_states: HashMap<String, ConsoleState>,          // console_id -> ConsoleState
    games: HashMap<String, Vec<Game>>,                        // console_id -> Vec<Game>
    consoles: Vec<Console>,
    ui_state: UiState,
}

#[derive(Default)]
struct UiState {
    active_tab: Tab,
    sort_by: SortOption,
    filter_by: FilterOption,
    search_query: String,
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
            self.game_states = load_all_game_states();
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
                                    
                                    // Merge imported game states
                                    for console_export in import.consoles {
                                        let console_states = self.game_states
                                            .entry(console_export.console_id.clone())
                                            .or_insert_with(std::collections::HashMap::new);
                                        
                                        for game_state in console_export.games {
                                            console_states.insert(game_state.game_id.clone(), game_state);
                                        }
                                    }
                                    
                                    // Save all imported states
                                    crate::persistence::save_console_states(&self.console_states);
                                    for (console_id, states) in &self.game_states {
                                        crate::persistence::save_game_states(console_id, states);
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
                Tab::Consoles => render_consoles_tab(ui, &self.consoles, &mut self.console_states, &self.game_states),
                Tab::Games => render_games_tab(
                    ui,
                    &mut self.selected_console,
                    &self.consoles,
                    &mut self.games,
                    &mut self.game_states,
                    &mut self.ui_state,
                ),
            }
        });
    }
}