use crate::{
    Console, ConsoleState, FilterOption, Game, GameState, SortOption, UiState,
};
use egui::*;
use std::collections::HashMap;

pub fn render_consoles_tab(
    ui: &mut egui::Ui,
    consoles: &[Console],
    console_states: &mut HashMap<String, ConsoleState>,
    game_states: &HashMap<String, HashMap<String, GameState>>,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Consoles");
        ui.separator();

        let mut needs_save = false;

        for console in consoles {
            // Get or create console state
            let console_state = console_states
                .entry(console.id.clone())
                .or_insert_with(|| ConsoleState {
                    console_id: console.id.clone(),
                    ..Default::default()
                });

            // Count games for this console
            let states = game_states.get(&console.id).map(|s| s.values()).unwrap_or_default();
            let owned_count = states.clone().filter(|s| s.owned).count();
            let favorite_count = states.clone().filter(|s| s.favorite).count();
            let wishlist_count = states.clone().filter(|s| s.wishlist).count();

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(&console.name);
                        ui.label(format!("Manufacturer: {}", console.manufacturer));
                        ui.label(format!("Year: {}", console.year));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.vertical(|ui| {
                            // Console state checkboxes
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut console_state.owned, "Owned").changed() {
                                    needs_save = true;
                                }
                                if ui.checkbox(&mut console_state.favorite, "Favorite").changed() {
                                    needs_save = true;
                                }
                                if ui.checkbox(&mut console_state.wishlist, "Wishlist").changed() {
                                    needs_save = true;
                                }
                            });
                            ui.separator();
                            // Game counts
                            ui.label(format!("Games - Owned: {}", owned_count));
                            ui.label(format!("Favorites: {}", favorite_count));
                            ui.label(format!("Wishlist: {}", wishlist_count));
                        });
                    });
                });

                ui.separator();

                ui.label("Notes:");
                if ui.text_edit_multiline(&mut console_state.notes).changed() {
                    needs_save = true;
                }
            });
            ui.add_space(10.0);
        }

        if needs_save {
            crate::persistence::save_console_states(console_states);
        }
    });
}

pub fn render_games_tab(
    ui: &mut egui::Ui,
    selected_console: &mut Option<String>,
    consoles: &[Console],
    games: &mut HashMap<String, Vec<Game>>,
    game_states: &mut HashMap<String, HashMap<String, GameState>>,
    ui_state: &mut UiState,
) {
    ui.horizontal(|ui| {
        ui.label("Select Console:");
        egui::ComboBox::from_id_source("console_select")
            .selected_text(
                selected_console
                    .as_ref()
                    .map(|id| {
                        consoles
                            .iter()
                            .find(|c| &c.id == id)
                            .map(|c| c.name.clone())
                            .unwrap_or_else(|| id.clone())
                    })
                    .unwrap_or_else(|| "None".to_string()),
            )
            .show_ui(ui, |ui| {
                for console in consoles {
                    if ui
                        .selectable_label(
                            selected_console.as_ref() == Some(&console.id),
                            &console.name,
                        )
                        .clicked()
                    {
                        *selected_console = Some(console.id.clone());
                    }
                }
            });
    });

    if let Some(console_id) = selected_console {
        ui.separator();

        // Controls
        ui.horizontal(|ui| {
            ui.label("Sort by:");
            ui.selectable_value(&mut ui_state.sort_by, SortOption::Title, "Title");
            ui.selectable_value(&mut ui_state.sort_by, SortOption::Year, "Year");
            ui.selectable_value(&mut ui_state.sort_by, SortOption::Status, "Status");

            ui.separator();

            ui.label("Filter:");
            ui.selectable_value(&mut ui_state.filter_by, FilterOption::All, "All");
            ui.selectable_value(&mut ui_state.filter_by, FilterOption::Owned, "Owned");
            ui.selectable_value(&mut ui_state.filter_by, FilterOption::Favorites, "Favorites");
            ui.selectable_value(
                &mut ui_state.filter_by,
                FilterOption::Wishlist,
                "Wishlist",
            );
            ui.selectable_value(
                &mut ui_state.filter_by,
                FilterOption::NotOwned,
                "Not Owned",
            );

            ui.separator();

            ui.label("Search:");
            ui.text_edit_singleline(&mut ui_state.search_query);
        });

        ui.separator();

        // Get games for this console
        let console_games = games.get(console_id).cloned().unwrap_or_default();
        let states = game_states
            .entry(console_id.clone())
            .or_insert_with(HashMap::new);

        // Initialize states for games that don't have one yet
        for game in &console_games {
            if !states.contains_key(&game.id) {
                states.insert(game.id.clone(), GameState {
                    game_id: game.id.clone(),
                    ..Default::default()
                });
            }
        }

        // Filter games
        let filtered_games: Vec<&Game> = console_games
            .iter()
            .filter(|game| {
                let state = states.get(&game.id).unwrap();
                
                // Search filter
                if !ui_state.search_query.is_empty() {
                    if !game.title.to_lowercase().contains(&ui_state.search_query.to_lowercase()) {
                        return false;
                    }
                }

                // Status filter
                match ui_state.filter_by {
                    FilterOption::All => true,
                    FilterOption::Owned => state.owned,
                    FilterOption::Favorites => state.favorite,
                    FilterOption::Wishlist => state.wishlist,
                    FilterOption::NotOwned => !state.owned,
                }
            })
            .collect();

        // Sort games
        let mut filtered_games: Vec<&Game> = filtered_games;
        match ui_state.sort_by {
            SortOption::Title => {
                filtered_games.sort_by(|a, b| a.title.cmp(&b.title));
            }
            SortOption::Year => {
                filtered_games.sort_by(|a, b| a.year.cmp(&b.year));
            }
            SortOption::Status => {
                filtered_games.sort_by(|a, b| {
                    let a_state = states.get(&a.id).unwrap();
                    let b_state = states.get(&b.id).unwrap();
                    // Sort by owned > favorite > wishlist > none
                    let a_priority = if a_state.owned { 3 } else if a_state.favorite { 2 } else if a_state.wishlist { 1 } else { 0 };
                    let b_priority = if b_state.owned { 3 } else if b_state.favorite { 2 } else if b_state.wishlist { 1 } else { 0 };
                    b_priority.cmp(&a_priority)
                });
            }
        }

        // Render games
        ScrollArea::vertical().show(ui, |ui| {
            let mut needs_save = false;
            
            for game in filtered_games {
                let state = states.get_mut(&game.id).unwrap();
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(&game.title);
                            ui.label(format!("Publisher: {}", game.publisher));
                            ui.label(format!("Year: {}", game.year));
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.vertical(|ui| {
                                if ui.checkbox(&mut state.owned, "Owned").changed() {
                                    needs_save = true;
                                }
                                if ui.checkbox(&mut state.favorite, "Favorite").changed() {
                                    needs_save = true;
                                }
                                if ui.checkbox(&mut state.wishlist, "Wishlist").changed() {
                                    needs_save = true;
                                }
                            });
                        });
                    });

                    ui.separator();

                    ui.label("Notes:");
                    if ui.text_edit_multiline(&mut state.notes).changed() {
                        needs_save = true;
                    }
                });

                ui.add_space(10.0);
            }
            
            if needs_save {
                crate::persistence::save_game_states(console_id, states);
            }
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Please select a console to view games.");
        });
    }
}
