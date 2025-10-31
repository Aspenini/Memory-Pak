use crate::{
    ConsoleExportData, ConsoleState, ExportData, GameState, MemoryPakApp,
};
use directories::ProjectDirs;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{Document, HtmlInputElement, Window};

pub fn get_state_dir() -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(proj_dirs) = ProjectDirs::from("com", "memorypak", "memory_pak") {
            let state_dir = proj_dirs.data_dir().join("state");
            if let Err(_) = fs::create_dir_all(&state_dir) {
                return None;
            }
            return Some(state_dir);
        }
        None
    }

    #[cfg(target_arch = "wasm32")]
    {
        None // Web uses localStorage, not file paths
    }
}

pub fn get_state_file_path(console_id: &str) -> Option<PathBuf> {
    get_state_dir().map(|dir| dir.join(format!("{}.json", console_id)))
}

pub fn load_game_states(console_id: &str) -> HashMap<String, GameState> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_state_file_path(console_id) {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(states) = serde_json::from_str::<Vec<GameState>>(&content) {
                    return states
                        .into_iter()
                        .map(|state| (state.game_id.clone(), state))
                        .collect();
                }
            }
        }
        HashMap::new()
    }

    #[cfg(target_arch = "wasm32")]
    {
        load_game_states_web(console_id)
    }
}

pub fn save_game_states(console_id: &str, states: &HashMap<String, GameState>) -> bool {
    let states_vec: Vec<GameState> = states.values().cloned().collect();
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_state_file_path(console_id) {
            if let Ok(json) = serde_json::to_string_pretty(&states_vec) {
                return fs::write(&path, json).is_ok();
            }
        }
        false
    }

    #[cfg(target_arch = "wasm32")]
    {
        save_game_states_web(console_id, &states_vec)
    }
}

pub fn load_all_game_states() -> HashMap<String, HashMap<String, GameState>> {
    let mut all_states = HashMap::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(state_dir) = get_state_dir() {
            if let Ok(entries) = fs::read_dir(&state_dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                        all_states.insert(file_name.to_string(), load_game_states(file_name));
                    }
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Load all console states from localStorage
        // We'll scan localStorage for keys matching our pattern
        if let Some(window) = web_sys::window() {
            if let Some(local_storage) = window.local_storage().ok().flatten() {
                // Try to get all keys - JavaScript doesn't give us a direct way,
                // so we'll try common console IDs and also check if there's a master list
                // For now, we'll check a comprehensive list of possible console IDs
                let possible_consoles = [
                    "nes", "snes", "n64", "gamecube", "wii", "wiiu", "switch",
                    "gb", "gba", "ds", "3ds",
                    "genesis", "saturn", "dreamcast",
                    "ps1", "ps2", "ps3", "ps4", "ps5",
                    "xbox", "xbox360", "xboxone", "xboxseries",
                ];
                
                for console_id in &possible_consoles {
                    let states = load_game_states_web(console_id);
                    if !states.is_empty() {
                        all_states.insert(console_id.to_string(), states);
                    }
                }
            }
        }
    }

    all_states
}

pub fn get_console_states_file_path() -> Option<PathBuf> {
    get_state_dir().map(|dir| dir.join("consoles.json"))
}

pub fn load_all_console_states() -> HashMap<String, ConsoleState> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_console_states_file_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(states_vec) = serde_json::from_str::<Vec<ConsoleState>>(&content) {
                    return states_vec
                        .into_iter()
                        .map(|state| (state.console_id.clone(), state))
                        .collect();
                }
            }
        }
        HashMap::new()
    }

    #[cfg(target_arch = "wasm32")]
    {
        load_console_states_web()
    }
}

pub fn save_console_states(states: &HashMap<String, ConsoleState>) -> bool {
    let states_vec: Vec<ConsoleState> = states.values().cloned().collect();
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_console_states_file_path() {
            if let Ok(json) = serde_json::to_string_pretty(&states_vec) {
                return fs::write(&path, json).is_ok();
            }
        }
        false
    }

    #[cfg(target_arch = "wasm32")]
    {
        save_console_states_web(&states_vec)
    }
}

#[cfg(target_arch = "wasm32")]
fn load_console_states_web() -> HashMap<String, ConsoleState> {
    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            if let Ok(Some(data)) = local_storage.get_item("memory_pak_console_states") {
                if let Ok(states) = serde_json::from_str::<Vec<ConsoleState>>(&data) {
                    return states
                        .into_iter()
                        .map(|state| (state.console_id.clone(), state))
                        .collect();
                }
            }
        }
    }
    HashMap::new()
}

#[cfg(target_arch = "wasm32")]
fn save_console_states_web(states: &Vec<ConsoleState>) -> bool {
    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            if let Ok(json) = serde_json::to_string(states) {
                return local_storage
                    .set_item("memory_pak_console_states", &json)
                    .is_ok();
            }
        }
    }
    false
}

#[cfg(target_arch = "wasm32")]
fn load_game_states_web(console_id: &str) -> HashMap<String, GameState> {
    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            if let Ok(Some(data)) = local_storage.get_item(&format!("memory_pak_state_{}", console_id)) {
                if let Ok(states) = serde_json::from_str::<Vec<GameState>>(&data) {
                    return states
                        .into_iter()
                        .map(|state| (state.game_id.clone(), state))
                        .collect();
                }
            }
        }
    }
    HashMap::new()
}

#[cfg(target_arch = "wasm32")]
fn save_game_states_web(console_id: &str, states: &Vec<GameState>) -> bool {
    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            if let Ok(json) = serde_json::to_string(states) {
                return local_storage
                    .set_item(&format!("memory_pak_state_{}", console_id), &json)
                    .is_ok();
            }
        }
    }
    false
}

pub fn export_data(app: &MemoryPakApp) -> Result<(), Box<dyn std::error::Error>> {
    let export = ExportData {
        version: "1.0".to_string(),
        export_date: chrono::Utc::now().to_rfc3339(),
        console_states: app.console_states.values().cloned().collect(),
        consoles: app
            .game_states
            .iter()
            .map(|(console_id, states)| ConsoleExportData {
                console_id: console_id.clone(),
                games: states.values().cloned().collect(),
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&export)?;

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use native file dialog
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("memory_pak_export.json")
            .save_file()
        {
            std::fs::write(path, json)?;
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Download via blob
        download_json_web(&json, "memory_pak_export.json");
    }
    
    Ok(())
}

pub fn import_data(app: &mut MemoryPakApp) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            let content = std::fs::read_to_string(path)?;
            let import: ExportData = serde_json::from_str(&content)?;
            
            // Merge imported console states
            for console_state in import.console_states {
                app.console_states.insert(console_state.console_id.clone(), console_state);
            }
            
            // Merge imported game states
            for console_export in import.consoles {
                let console_states = app.game_states
                    .entry(console_export.console_id.clone())
                    .or_insert_with(HashMap::new);
                
                for game_state in console_export.games {
                    console_states.insert(game_state.game_id.clone(), game_state);
                }
            }
            
            // Save all imported states
            save_console_states(&app.console_states);
            for (console_id, states) in &app.game_states {
                save_game_states(console_id, states);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // For web, we'll show a modal dialog for importing JSON text
        // The UI will handle showing the import dialog
        app.ui_state.needs_import = true;
        app.ui_state.import_text.clear();
    }
    
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn download_json_web(json: &str, filename: &str) {
    if let Some(window) = web_sys::window() {
        let document = window.document().unwrap();
        let blob = js_sys::Blob::new_with_str_sequence(
            &js_sys::Array::of1(&js_sys::JsString::from(json)),
        )
        .unwrap();
        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

        let a = document.create_element("a").unwrap();
        let a = a.dyn_into::<web_sys::HtmlElement>().unwrap();
        a.set_attribute("href", &url).unwrap();
        a.set_attribute("download", filename).unwrap();
        a.style().set_property("display", "none").unwrap();
        document.body().unwrap().append_child(&a).unwrap();
        a.click();
        document.body().unwrap().remove_child(&a).unwrap();
        web_sys::Url::revoke_object_url(&url).unwrap();
    }
}

