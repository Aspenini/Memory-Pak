use crate::game_data::get_console_from_id;
use crate::lego_dimensions::{figure_id, LegoDimensionFigure, LegoDimensionState};
use crate::{
    Console, ConsoleState, FilterOption, Game, GameState, LegoSortOption, SortOption, UiState,
};
use egui::*;
use std::collections::HashMap;

fn console_display_name(console: &Console) -> String {
    if let Some(variant) = &console.variant {
        format!("{} ({})", console.name, variant)
    } else {
        console.name.clone()
    }
}

pub fn render_consoles_tab(
    ui: &mut egui::Ui,
    consoles: &[Console],
    console_states: &mut HashMap<String, ConsoleState>,
    game_states: &HashMap<String, GameState>,
    ui_state: &mut UiState,
) {
    ui.heading("Consoles");
    ui.separator();

    // Controls
    ui.horizontal(|ui| {
        ui.label("Sort by:");
        ui.selectable_value(&mut ui_state.console_sort_by, SortOption::Title, "Title");
        ui.selectable_value(&mut ui_state.console_sort_by, SortOption::Year, "Year");
        ui.selectable_value(&mut ui_state.console_sort_by, SortOption::Status, "Status");

        ui.separator();

        ui.label("Filter:");
        ui.selectable_value(&mut ui_state.console_filter_by, FilterOption::All, "All");
        ui.selectable_value(
            &mut ui_state.console_filter_by,
            FilterOption::Owned,
            "Owned",
        );
        ui.selectable_value(
            &mut ui_state.console_filter_by,
            FilterOption::Favorites,
            "Favorites",
        );
        ui.selectable_value(
            &mut ui_state.console_filter_by,
            FilterOption::Wishlist,
            "Wishlist",
        );
        ui.selectable_value(
            &mut ui_state.console_filter_by,
            FilterOption::NotOwned,
            "Not Owned",
        );
    });

    ui.separator();

    // Filter and sort consoles
    let mut filtered_consoles: Vec<&Console> = consoles.iter().collect();

    // Apply filter
    filtered_consoles.retain(|console| {
        let state = console_states.get(&console.id);

        match ui_state.console_filter_by {
            FilterOption::All => true,
            FilterOption::Owned => state.map(|s| s.owned).unwrap_or(false),
            FilterOption::Favorites => state.map(|s| s.favorite).unwrap_or(false),
            FilterOption::Wishlist => state.map(|s| s.wishlist).unwrap_or(false),
            FilterOption::NotOwned => !state.map(|s| s.owned).unwrap_or(false),
        }
    });

    // Sort consoles
    filtered_consoles.sort_by(|a, b| {
        let a_state = console_states.get(&a.id);
        let b_state = console_states.get(&b.id);

        match ui_state.console_sort_by {
            SortOption::Title => a.name.cmp(&b.name),
            SortOption::Year => a.year.cmp(&b.year),
            SortOption::Status => {
                // Sort by status priority: owned > favorite > wishlist > none
                let a_priority = if let Some(state) = a_state {
                    if state.owned {
                        3
                    } else if state.favorite {
                        2
                    } else if state.wishlist {
                        1
                    } else {
                        0
                    }
                } else {
                    0
                };
                let b_priority = if let Some(state) = b_state {
                    if state.owned {
                        3
                    } else if state.favorite {
                        2
                    } else if state.wishlist {
                        1
                    } else {
                        0
                    }
                } else {
                    0
                };
                b_priority.cmp(&a_priority)
            }
        }
    });

    // Pagination
    const CONSOLES_PER_PAGE: usize = 20;
    let total_consoles = filtered_consoles.len();
    let total_pages = if total_consoles == 0 {
        1
    } else {
        (total_consoles + CONSOLES_PER_PAGE - 1) / CONSOLES_PER_PAGE
    };

    if ui_state.consoles_page >= total_pages {
        ui_state.consoles_page = 0;
    }

    let start_idx = ui_state.consoles_page * CONSOLES_PER_PAGE;
    let end_idx = (start_idx + CONSOLES_PER_PAGE).min(total_consoles);
    let consoles_for_page = &filtered_consoles[start_idx..end_idx];

    ui.label(format!(
        "Showing {} of {} consoles (Page {} of {})",
        consoles_for_page.len(),
        total_consoles,
        ui_state.consoles_page + 1,
        total_pages
    ));

    ui.separator();

    // Use vertical layout to ensure pagination is always at bottom
    ui.vertical(|ui| {
        // Calculate available space
        let pagination_height = if total_pages > 1 { 80.0 } else { 0.0 };
        let available = ui.available_size();
        let scroll_height = available.y - pagination_height;

        // Render consoles in scroll area with constrained height
        ScrollArea::vertical()
            .max_height(scroll_height.max(100.0))
            .show(ui, |ui| {
                let mut needs_save = false;

                for console in consoles_for_page {
                    // Get or create console state
                    let console_state =
                        console_states
                            .entry(console.id.clone())
                            .or_insert_with(|| ConsoleState {
                                console_id: console.id.clone(),
                                ..Default::default()
                            });

                    // Count games for this console
                    let owned_count = game_states
                        .values()
                        .filter(|state| {
                            if let Some(game_console) = state.game_id.split('-').next() {
                                game_console == console.id
                            } else {
                                false
                            }
                        })
                        .filter(|state| state.owned)
                        .count();

                    let favorite_count = game_states
                        .values()
                        .filter(|state| {
                            if let Some(game_console) = state.game_id.split('-').next() {
                                game_console == console.id
                            } else {
                                false
                            }
                        })
                        .filter(|state| state.favorite)
                        .count();

                    let wishlist_count = game_states
                        .values()
                        .filter(|state| {
                            if let Some(game_console) = state.game_id.split('-').next() {
                                game_console == console.id
                            } else {
                                false
                            }
                        })
                        .filter(|state| state.wishlist)
                        .count();

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                let display_name = console_display_name(console);
                                ui.heading(&display_name);
                                ui.label(format!("Manufacturer: {}", console.manufacturer));
                                ui.label(format!("Year: {}", console.year));
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.vertical(|ui| {
                                    // Console state checkboxes
                                    ui.horizontal(|ui| {
                                        if ui.checkbox(&mut console_state.owned, "Owned").changed()
                                        {
                                            needs_save = true;
                                        }
                                        if ui
                                            .checkbox(&mut console_state.favorite, "Favorite")
                                            .changed()
                                        {
                                            needs_save = true;
                                        }
                                        if ui
                                            .checkbox(&mut console_state.wishlist, "Wishlist")
                                            .changed()
                                        {
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

        // Pagination controls - ALWAYS RENDERED at bottom
        if total_pages > 1 {
            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 0.0);

                // Previous button
                if ui.button("◀ Previous").clicked() && ui_state.consoles_page > 0 {
                    ui_state.consoles_page -= 1;
                }

                // Page number buttons (show up to 10 pages, centered around current)
                let pages_to_show = 10;
                let start_page = if total_pages <= pages_to_show {
                    0
                } else if ui_state.consoles_page < pages_to_show / 2 {
                    0
                } else if ui_state.consoles_page >= total_pages - pages_to_show / 2 {
                    total_pages.saturating_sub(pages_to_show)
                } else {
                    ui_state.consoles_page - pages_to_show / 2
                };

                let end_page = (start_page + pages_to_show).min(total_pages);

                if start_page > 0 {
                    if ui.button("1").clicked() {
                        ui_state.consoles_page = 0;
                    }
                    if start_page > 1 {
                        ui.label("...");
                    }
                }

                for page_num in start_page..end_page {
                    let is_current = page_num == ui_state.consoles_page;
                    let button_text = format!("{}", page_num + 1);
                    if ui.selectable_label(is_current, button_text).clicked() {
                        ui_state.consoles_page = page_num;
                    }
                }

                if end_page < total_pages {
                    if end_page < total_pages - 1 {
                        ui.label("...");
                    }
                    if ui.button(format!("{}", total_pages)).clicked() {
                        ui_state.consoles_page = total_pages - 1;
                    }
                }

                // Next button
                if ui.button("Next ▶").clicked() && ui_state.consoles_page < total_pages - 1 {
                    ui_state.consoles_page += 1;
                }
            });
        }
    });
}

pub fn render_games_tab(
    ui: &mut egui::Ui,
    selected_console: &mut Option<String>,
    consoles: &[Console],
    games: &HashMap<String, Game>,
    game_states: &mut HashMap<String, GameState>,
    ui_state: &mut UiState,
) {
    ui.horizontal(|ui| {
        ui.label("Select Console:");
        egui::ComboBox::from_id_source("console_select")
            .selected_text(match selected_console.as_deref() {
                Some("all") => "All Consoles".to_string(),
                Some(id) => consoles
                    .iter()
                    .find(|c| &c.id == id)
                    .map(|c| console_display_name(c))
                    .unwrap_or_else(|| id.to_string()),
                None => "None".to_string(),
            })
            .show_ui(ui, |ui| {
                // Add "All Consoles" option
                if ui
                    .selectable_label(selected_console.as_deref() == Some("all"), "All Consoles")
                    .clicked()
                {
                    *selected_console = Some("all".to_string());
                }

                for console in consoles {
                    if ui
                        .selectable_label(
                            selected_console.as_deref() == Some(&console.id),
                            console_display_name(console),
                        )
                        .clicked()
                    {
                        *selected_console = Some(console.id.clone());
                    }
                }
            });
    });

    let console_filter = selected_console.as_deref().unwrap_or("");

    if !console_filter.is_empty() {
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
            ui.selectable_value(
                &mut ui_state.filter_by,
                FilterOption::Favorites,
                "Favorites",
            );
            ui.selectable_value(&mut ui_state.filter_by, FilterOption::Wishlist, "Wishlist");
            ui.selectable_value(&mut ui_state.filter_by, FilterOption::NotOwned, "Not Owned");

            ui.separator();

            ui.label("Search:");
            ui.text_edit_singleline(&mut ui_state.search_query);
        });

        ui.separator();

        // Check if filters changed and reset page if needed
        // We'll use a simple approach: reset page when console changes
        // The page bounds check below will handle other filter changes
        static mut LAST_CONSOLE_FILTER: Option<String> = None;
        unsafe {
            let current_filter = console_filter.to_string();
            if let Some(ref last_filter) = LAST_CONSOLE_FILTER {
                if last_filter != &current_filter {
                    ui_state.games_page = 0;
                }
            }
            LAST_CONSOLE_FILTER = Some(current_filter);
        }

        // Filter games by console (if not "all")
        let games_to_show: Vec<&Game> = if console_filter == "all" {
            games.values().collect()
        } else {
            games
                .values()
                .filter(|game| game.console_id == console_filter)
                .collect()
        };

        // Initialize states for games that don't have one yet
        for game in &games_to_show {
            if !game_states.contains_key(&game.id) {
                game_states.insert(
                    game.id.clone(),
                    GameState {
                        game_id: game.id.clone(),
                        ..Default::default()
                    },
                );
            }
        }

        // Filter games
        let filtered_games: Vec<&Game> = games_to_show
            .into_iter()
            .filter(|game| {
                let state = game_states.get(&game.id).unwrap();

                // Search filter
                if !ui_state.search_query.is_empty() {
                    if !game
                        .title
                        .to_lowercase()
                        .contains(&ui_state.search_query.to_lowercase())
                    {
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
                    let a_state = game_states.get(&a.id).unwrap();
                    let b_state = game_states.get(&b.id).unwrap();
                    // Sort by owned > favorite > wishlist > none
                    let a_priority = if a_state.owned {
                        3
                    } else if a_state.favorite {
                        2
                    } else if a_state.wishlist {
                        1
                    } else {
                        0
                    };
                    let b_priority = if b_state.owned {
                        3
                    } else if b_state.favorite {
                        2
                    } else if b_state.wishlist {
                        1
                    } else {
                        0
                    };
                    b_priority.cmp(&a_priority)
                });
            }
        }

        // Pagination
        const GAMES_PER_PAGE: usize = 50;
        let total_games = filtered_games.len();
        let total_pages = if total_games == 0 {
            1
        } else {
            (total_games + GAMES_PER_PAGE - 1) / GAMES_PER_PAGE
        };

        // Reset to page 0 if current page is out of bounds
        if ui_state.games_page >= total_pages {
            ui_state.games_page = 0;
        }

        // Get games for current page
        let start_idx = ui_state.games_page * GAMES_PER_PAGE;
        let end_idx = (start_idx + GAMES_PER_PAGE).min(total_games);
        let games_for_page = &filtered_games[start_idx..end_idx];

        // Show game count and page info
        ui.label(format!(
            "Showing {} of {} games (Page {} of {})",
            games_for_page.len(),
            total_games,
            ui_state.games_page + 1,
            total_pages
        ));

        ui.separator();

        // Use vertical layout to ensure pagination is always at bottom
        ui.vertical(|ui| {
            // Calculate available space
            let pagination_height = if total_pages > 1 { 80.0 } else { 0.0 };
            let available = ui.available_size();
            let scroll_height = available.y - pagination_height;

            // Render games in scroll area with constrained height
            egui::ScrollArea::vertical()
                .max_height(scroll_height.max(100.0))
                .show(ui, |ui| {
                    let mut needs_save = false;

                    for game in games_for_page {
                        let state = game_states.get_mut(&game.id).unwrap();

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    // Show console badge if viewing "all" consoles
                                    if console_filter == "all" {
                                        let console_name = consoles
                                            .iter()
                                            .find(|c| c.id == game.console_id)
                                            .map(|c| console_display_name(c))
                                            .unwrap_or_else(|| game.console_id.clone());
                                        ui.label(format!("[{}]", console_name));
                                    }
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
                        // Save states grouped by console
                        let mut states_by_console: HashMap<String, HashMap<String, GameState>> =
                            HashMap::new();
                        for (game_id, state) in game_states.iter() {
                            let console_id = get_console_from_id(game_id);
                            states_by_console
                                .entry(console_id.to_string())
                                .or_insert_with(HashMap::new)
                                .insert(game_id.clone(), state.clone());
                        }

                        for (console_id, console_states) in states_by_console {
                            crate::persistence::save_game_states(&console_id, &console_states);
                        }
                    }
                });

            // Pagination controls - ALWAYS RENDERED at bottom
            if total_pages > 1 {
                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(10.0, 0.0);

                    // Previous button
                    if ui.button("◀ Previous").clicked() && ui_state.games_page > 0 {
                        ui_state.games_page -= 1;
                    }

                    // Page number buttons (show up to 10 pages, centered around current)
                    let pages_to_show = 10;
                    let start_page = if total_pages <= pages_to_show {
                        0
                    } else if ui_state.games_page < pages_to_show / 2 {
                        0
                    } else if ui_state.games_page >= total_pages - pages_to_show / 2 {
                        total_pages.saturating_sub(pages_to_show)
                    } else {
                        ui_state.games_page - pages_to_show / 2
                    };

                    let end_page = (start_page + pages_to_show).min(total_pages);

                    if start_page > 0 {
                        if ui.button("1").clicked() {
                            ui_state.games_page = 0;
                        }
                        if start_page > 1 {
                            ui.label("...");
                        }
                    }

                    for page_num in start_page..end_page {
                        let is_current = page_num == ui_state.games_page;
                        let button_text = format!("{}", page_num + 1);
                        if ui.selectable_label(is_current, button_text).clicked() {
                            ui_state.games_page = page_num;
                        }
                    }

                    if end_page < total_pages {
                        if end_page < total_pages - 1 {
                            ui.label("...");
                        }
                        if ui.button(format!("{}", total_pages)).clicked() {
                            ui_state.games_page = total_pages - 1;
                        }
                    }

                    // Next button
                    if ui.button("Next ▶").clicked() && ui_state.games_page < total_pages - 1 {
                        ui_state.games_page += 1;
                    }
                });
            }
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Please select a console to view games.");
        });
    }
}

pub fn render_lego_dimensions_tab(
    ui: &mut egui::Ui,
    figures: &[LegoDimensionFigure],
    states: &mut HashMap<String, LegoDimensionState>,
    ui_state: &mut UiState,
) {
    ui.heading("LEGO Dimensions Characters");
    ui.separator();

    if figures.is_empty() {
        ui.label("No LEGO Dimensions data available.");
        return;
    }

    // Ensure states exist for all figures
    for figure in figures {
        let id = figure_id(figure);
        states
            .entry(id.clone())
            .or_insert_with(|| LegoDimensionState {
                figure_id: id,
                ..Default::default()
            });
    }

    ui.horizontal(|ui| {
        ui.label("Sort by:");
        ui.selectable_value(&mut ui_state.lego_sort_by, LegoSortOption::Name, "Name");
        ui.selectable_value(
            &mut ui_state.lego_sort_by,
            LegoSortOption::Category,
            "Category",
        );
        ui.selectable_value(&mut ui_state.lego_sort_by, LegoSortOption::Year, "Year");
        ui.selectable_value(&mut ui_state.lego_sort_by, LegoSortOption::Pack, "Pack");

        ui.separator();

        ui.label("Filter:");
        ui.selectable_value(&mut ui_state.lego_filter_by, FilterOption::All, "All");
        ui.selectable_value(&mut ui_state.lego_filter_by, FilterOption::Owned, "Owned");
        ui.selectable_value(
            &mut ui_state.lego_filter_by,
            FilterOption::Favorites,
            "Favorites",
        );
        ui.selectable_value(
            &mut ui_state.lego_filter_by,
            FilterOption::Wishlist,
            "Wishlist",
        );
        ui.selectable_value(
            &mut ui_state.lego_filter_by,
            FilterOption::NotOwned,
            "Not Owned",
        );

        ui.separator();

        ui.label("Search:");
        ui.text_edit_singleline(&mut ui_state.lego_search_query);
    });

    ui.separator();

    let mut filtered_figures: Vec<&LegoDimensionFigure> = figures
        .iter()
        .filter(|figure| {
            let figure_state = states
                .get(&figure_id(figure))
                .expect("state should exist for figure");

            if !ui_state.lego_search_query.is_empty()
                && !figure
                    .name
                    .to_lowercase()
                    .contains(&ui_state.lego_search_query.to_lowercase())
                && !figure
                    .category
                    .to_lowercase()
                    .contains(&ui_state.lego_search_query.to_lowercase())
                && !figure
                    .pack_id
                    .to_lowercase()
                    .contains(&ui_state.lego_search_query.to_lowercase())
            {
                return false;
            }

            match ui_state.lego_filter_by {
                FilterOption::All => true,
                FilterOption::Owned => figure_state.owned,
                FilterOption::Favorites => figure_state.favorite,
                FilterOption::Wishlist => figure_state.wishlist,
                FilterOption::NotOwned => !figure_state.owned,
            }
        })
        .collect();

    match ui_state.lego_sort_by {
        LegoSortOption::Name => filtered_figures.sort_by(|a, b| a.name.cmp(&b.name)),
        LegoSortOption::Category => filtered_figures.sort_by(|a, b| {
            a.category
                .cmp(&b.category)
                .then(a.name.cmp(&b.name))
                .then(a.pack_id.cmp(&b.pack_id))
        }),
        LegoSortOption::Year => filtered_figures.sort_by(|a, b| {
            a.year
                .cmp(&b.year)
                .then(a.category.cmp(&b.category))
                .then(a.name.cmp(&b.name))
        }),
        LegoSortOption::Pack => filtered_figures.sort_by(|a, b| {
            a.pack_id
                .cmp(&b.pack_id)
                .then(a.category.cmp(&b.category))
                .then(a.name.cmp(&b.name))
        }),
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let mut needs_save = false;

            for figure in filtered_figures {
                let id = figure_id(figure);
                let state = states
                    .get_mut(&id)
                    .expect("state should exist for figure when rendering");

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(&figure.name);
                            ui.label(format!("Category: {}", figure.category));
                            ui.label(format!("Pack ID: {}", figure.pack_id));
                            ui.label(format!("Year: {}", figure.year));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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

                ui.add_space(8.0);
            }

            if needs_save {
                crate::persistence::save_lego_dimensions_states(states);
            }
        });
}
