use crate::{
    ConsoleExportData, ConsoleState, ExportData, GameState, LegoDimensionState, MemoryPakApp,
    SkylanderState,
};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::io::ErrorKind;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(not(target_arch = "wasm32"))]
pub fn get_state_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "memorypak", "memory_pak") {
        let state_dir = proj_dirs.data_dir().join("state");
        if fs::create_dir_all(&state_dir).is_err() {
            return None;
        }
        return Some(state_dir);
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_state_file_path(console_id: &str) -> Option<PathBuf> {
    get_state_dir().map(|dir| dir.join(format!("{}.json", console_id)))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_lego_dimensions_state_file_path() -> Option<PathBuf> {
    get_state_dir().map(|dir| dir.join("lego_dimensions.json"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_skylanders_state_file_path() -> Option<PathBuf> {
    get_state_dir().map(|dir| dir.join("skylanders.json"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game_states(console_id: &str) -> HashMap<String, GameState> {
    if let Some(path) = get_state_file_path(console_id) {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Vec<GameState>>(&content) {
                Ok(states) => {
                    return states
                        .into_iter()
                        .map(|state| (state.game_id.clone(), state))
                        .collect();
                }
                Err(err) => {
                    eprintln!("Failed to parse game state file {}: {err}", path.display());
                }
            },
            Err(err) if err.kind() != ErrorKind::NotFound => {
                eprintln!("Failed to read game state file {}: {err}", path.display());
            }
            Err(_) => {}
        }
    }
    HashMap::new()
}

pub fn save_game_states(console_id: &str, states: &HashMap<String, GameState>) -> bool {
    let mut states_vec: Vec<GameState> = states.values().cloned().collect();
    states_vec.sort_by(|a, b| a.game_id.cmp(&b.game_id));

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

/// Load all game states into a flat HashMap (game_id -> GameState)
/// This consolidates all console-specific state files into one flat structure
pub fn load_all_game_states_flat() -> HashMap<String, GameState> {
    let mut flat_states = HashMap::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(state_dir) = get_state_dir() {
            if let Ok(entries) = fs::read_dir(&state_dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                        // Skip console states file
                        if file_name == "consoles" {
                            continue;
                        }
                        let console_states = load_game_states(file_name);
                        // Flatten into single HashMap
                        for (game_id, state) in console_states {
                            flat_states.insert(game_id, state);
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Load all console states from localStorage and flatten
        if let Some(window) = web_sys::window() {
            if window.local_storage().ok().flatten().is_some() {
                for console_id in crate::game_data::game_database_console_ids() {
                    let console_states = load_game_states_web(&console_id);
                    for (game_id, state) in console_states {
                        flat_states.insert(game_id, state);
                    }
                }
            }
        }
    }

    flat_states
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_console_states_file_path() -> Option<PathBuf> {
    get_state_dir().map(|dir| dir.join("consoles.json"))
}

pub fn load_all_console_states() -> HashMap<String, ConsoleState> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_console_states_file_path() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Vec<ConsoleState>>(&content) {
                    Ok(states_vec) => {
                        return states_vec
                            .into_iter()
                            .map(|state| (state.console_id.clone(), state))
                            .collect();
                    }
                    Err(err) => {
                        eprintln!(
                            "Failed to parse console state file {}: {err}",
                            path.display()
                        );
                    }
                },
                Err(err) if err.kind() != ErrorKind::NotFound => {
                    eprintln!(
                        "Failed to read console state file {}: {err}",
                        path.display()
                    );
                }
                Err(_) => {}
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
    let mut states_vec: Vec<ConsoleState> = states.values().cloned().collect();
    states_vec.sort_by(|a, b| a.console_id.cmp(&b.console_id));

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

pub fn load_lego_dimensions_states() -> HashMap<String, LegoDimensionState> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_lego_dimensions_state_file_path() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Vec<LegoDimensionState>>(&content) {
                    Ok(states_vec) => {
                        return states_vec
                            .into_iter()
                            .map(|state| (state.figure_id.clone(), state))
                            .collect();
                    }
                    Err(err) => {
                        eprintln!(
                            "Failed to parse LEGO Dimensions state file {}: {err}",
                            path.display()
                        );
                    }
                },
                Err(err) if err.kind() != ErrorKind::NotFound => {
                    eprintln!(
                        "Failed to read LEGO Dimensions state file {}: {err}",
                        path.display()
                    );
                }
                Err(_) => {}
            }
        }
        HashMap::new()
    }

    #[cfg(target_arch = "wasm32")]
    {
        load_lego_dimensions_states_web()
    }
}

pub fn save_lego_dimensions_states(states: &HashMap<String, LegoDimensionState>) -> bool {
    let mut states_vec: Vec<LegoDimensionState> = states.values().cloned().collect();
    states_vec.sort_by(|a, b| a.figure_id.cmp(&b.figure_id));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_lego_dimensions_state_file_path() {
            if let Ok(json) = serde_json::to_string_pretty(&states_vec) {
                return fs::write(&path, json).is_ok();
            }
        }
        false
    }

    #[cfg(target_arch = "wasm32")]
    {
        save_lego_dimensions_states_web(&states_vec)
    }
}

#[cfg(target_arch = "wasm32")]
fn load_console_states_web() -> HashMap<String, ConsoleState> {
    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            if let Ok(Some(data)) = local_storage.get_item("memory_pak_console_states") {
                match serde_json::from_str::<Vec<ConsoleState>>(&data) {
                    Ok(states) => {
                        return states
                            .into_iter()
                            .map(|state| (state.console_id.clone(), state))
                            .collect();
                    }
                    Err(err) => log_web_error(&format!("Failed to parse console states: {err}")),
                }
            }
        }
    }
    HashMap::new()
}

#[cfg(target_arch = "wasm32")]
fn save_console_states_web(states: &Vec<ConsoleState>) -> bool {
    use wasm_bindgen::JsValue;

    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            match serde_json::to_string(states) {
                Ok(json) => match local_storage.set_item("memory_pak_console_states", &json) {
                    Ok(_) => return true,
                    Err(e) => {
                        web_sys::console::error_1(&JsValue::from_str(&format!(
                            "Failed to save console states: {:?}",
                            e
                        )));
                        return false;
                    }
                },
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "Failed to serialize console states: {}",
                        e
                    )));
                }
            }
        }
    }
    false
}

#[cfg(target_arch = "wasm32")]
fn load_game_states_web(console_id: &str) -> HashMap<String, GameState> {
    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            if let Ok(Some(data)) =
                local_storage.get_item(&format!("memory_pak_state_{}", console_id))
            {
                match serde_json::from_str::<Vec<GameState>>(&data) {
                    Ok(states) => {
                        return states
                            .into_iter()
                            .map(|state| (state.game_id.clone(), state))
                            .collect();
                    }
                    Err(err) => {
                        log_web_error(&format!(
                            "Failed to parse game states for {console_id}: {err}"
                        ));
                    }
                }
            }
        }
    }
    HashMap::new()
}

#[cfg(target_arch = "wasm32")]
fn save_game_states_web(console_id: &str, states: &Vec<GameState>) -> bool {
    use wasm_bindgen::JsValue;

    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            match serde_json::to_string(states) {
                Ok(json) => {
                    match local_storage.set_item(&format!("memory_pak_state_{}", console_id), &json)
                    {
                        Ok(_) => return true,
                        Err(e) => {
                            web_sys::console::error_1(&JsValue::from_str(&format!(
                                "Failed to save game states for {}: {:?}",
                                console_id, e
                            )));
                            return false;
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "Failed to serialize game states: {}",
                        e
                    )));
                }
            }
        }
    }
    false
}

#[cfg(target_arch = "wasm32")]
fn load_lego_dimensions_states_web() -> HashMap<String, LegoDimensionState> {
    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            if let Ok(Some(data)) = local_storage.get_item("memory_pak_lego_dimensions_states") {
                match serde_json::from_str::<Vec<LegoDimensionState>>(&data) {
                    Ok(states) => {
                        return states
                            .into_iter()
                            .map(|state| (state.figure_id.clone(), state))
                            .collect();
                    }
                    Err(err) => {
                        log_web_error(&format!("Failed to parse LEGO Dimensions states: {err}"));
                    }
                }
            }
        }
    }
    HashMap::new()
}

#[cfg(target_arch = "wasm32")]
fn save_lego_dimensions_states_web(states: &Vec<LegoDimensionState>) -> bool {
    use wasm_bindgen::JsValue;

    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            match serde_json::to_string(states) {
                Ok(json) => {
                    match local_storage.set_item("memory_pak_lego_dimensions_states", &json) {
                        Ok(_) => return true,
                        Err(e) => {
                            web_sys::console::error_1(&JsValue::from_str(&format!(
                                "Failed to save LEGO Dimensions states: {:?}",
                                e
                            )));
                            return false;
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "Failed to serialize LEGO Dimensions states: {}",
                        e
                    )));
                }
            }
        }
    }
    false
}

pub fn load_skylanders_states() -> HashMap<String, SkylanderState> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_skylanders_state_file_path() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Vec<SkylanderState>>(&content) {
                    Ok(states_vec) => {
                        return states_vec
                            .into_iter()
                            .map(|state| (state.skylander_id.clone(), state))
                            .collect();
                    }
                    Err(err) => {
                        eprintln!(
                            "Failed to parse Skylanders state file {}: {err}",
                            path.display()
                        );
                    }
                },
                Err(err) if err.kind() != ErrorKind::NotFound => {
                    eprintln!(
                        "Failed to read Skylanders state file {}: {err}",
                        path.display()
                    );
                }
                Err(_) => {}
            }
        }
        HashMap::new()
    }

    #[cfg(target_arch = "wasm32")]
    {
        load_skylanders_states_web()
    }
}

pub fn save_skylanders_states(states: &HashMap<String, SkylanderState>) -> bool {
    let mut states_vec: Vec<SkylanderState> = states.values().cloned().collect();
    states_vec.sort_by(|a, b| a.skylander_id.cmp(&b.skylander_id));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = get_skylanders_state_file_path() {
            if let Ok(json) = serde_json::to_string_pretty(&states_vec) {
                return fs::write(&path, json).is_ok();
            }
        }
        false
    }

    #[cfg(target_arch = "wasm32")]
    {
        save_skylanders_states_web(&states_vec)
    }
}

#[cfg(target_arch = "wasm32")]
fn load_skylanders_states_web() -> HashMap<String, SkylanderState> {
    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            if let Ok(Some(data)) = local_storage.get_item("memory_pak_skylanders_states") {
                match serde_json::from_str::<Vec<SkylanderState>>(&data) {
                    Ok(states) => {
                        return states
                            .into_iter()
                            .map(|state| (state.skylander_id.clone(), state))
                            .collect();
                    }
                    Err(err) => {
                        log_web_error(&format!("Failed to parse Skylanders states: {err}"));
                    }
                }
            }
        }
    }
    HashMap::new()
}

#[cfg(target_arch = "wasm32")]
fn save_skylanders_states_web(states: &Vec<SkylanderState>) -> bool {
    use wasm_bindgen::JsValue;

    if let Some(window) = web_sys::window() {
        if let Some(local_storage) = window.local_storage().ok().flatten() {
            match serde_json::to_string(states) {
                Ok(json) => match local_storage.set_item("memory_pak_skylanders_states", &json) {
                    Ok(_) => return true,
                    Err(e) => {
                        web_sys::console::error_1(&JsValue::from_str(&format!(
                            "Failed to save Skylanders states: {:?}",
                            e
                        )));
                        return false;
                    }
                },
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "Failed to serialize Skylanders states: {}",
                        e
                    )));
                }
            }
        }
    }
    false
}

pub fn export_data(app: &MemoryPakApp) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "android")]
    {
        let json = export_json(app)?;
        let filename = format!(
            "memory_pak_export_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        crate::android_platform::export_json_to_downloads(&filename, &json)?;
        crate::android_platform::toast("Exported to Downloads/Memory Pak");
        Ok(())
    }

    #[cfg(not(target_os = "android"))]
    {
        let json = export_json(app)?;

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
            download_json_web(&json, "memory_pak_export.json")?;
        }

        Ok(())
    }
}

pub fn import_data(app: &mut MemoryPakApp) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "android")]
    {
        let _ = app;
        crate::android_platform::show_import_hint();
        Ok(())
    }

    #[cfg(not(target_os = "android"))]
    {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            let content = std::fs::read_to_string(path)?;
            let import: ExportData = serde_json::from_str(&content)?;
            apply_import_data(app, import);
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
}

pub fn apply_import_data(app: &mut MemoryPakApp, import: ExportData) {
    // Merge imported console states
    for console_state in import.console_states {
        app.console_states
            .insert(console_state.console_id.clone(), console_state);
    }

    // Merge imported game states (flat structure)
    for console_export in import.consoles {
        for game_state in console_export.games {
            app.game_states
                .insert(game_state.game_id.clone(), game_state);
        }
    }
    app.invalidate_game_counts_cache();

    for figure_state in import.lego_dimensions_states {
        app.lego_dimensions_states
            .insert(figure_state.figure_id.clone(), figure_state);
    }

    for skylander_state in import.skylanders_states {
        app.skylanders_states
            .insert(skylander_state.skylander_id.clone(), skylander_state);
    }

    save_console_states(&app.console_states);
    let mut states_by_console: HashMap<String, HashMap<String, GameState>> = HashMap::new();
    for (game_id, state) in &app.game_states {
        let console_id = crate::game_data::get_console_from_id(game_id).to_string();
        states_by_console
            .entry(console_id)
            .or_default()
            .insert(game_id.clone(), state.clone());
    }
    for (console_id, states) in states_by_console {
        save_game_states(&console_id, &states);
    }
    save_lego_dimensions_states(&app.lego_dimensions_states);
    save_skylanders_states(&app.skylanders_states);
}

pub(crate) fn export_json(app: &MemoryPakApp) -> Result<String, serde_json::Error> {
    // Group game states by console
    let mut games_by_console: HashMap<String, Vec<GameState>> = HashMap::new();
    for (game_id, state) in &app.game_states {
        let console_id = crate::game_data::get_console_from_id(game_id).to_string();
        if console_id.is_empty() {
            continue;
        }
        games_by_console
            .entry(console_id)
            .or_default()
            .push(state.clone());
    }

    let mut console_states: Vec<ConsoleState> = app.console_states.values().cloned().collect();
    console_states.sort_by(|a, b| a.console_id.cmp(&b.console_id));

    let mut consoles: Vec<ConsoleExportData> = games_by_console
        .into_iter()
        .map(|(console_id, mut games)| {
            games.sort_by(|a, b| a.game_id.cmp(&b.game_id));
            ConsoleExportData { console_id, games }
        })
        .collect();
    consoles.sort_by(|a, b| a.console_id.cmp(&b.console_id));

    let mut lego_dimensions_states: Vec<LegoDimensionState> =
        app.lego_dimensions_states.values().cloned().collect();
    lego_dimensions_states.sort_by(|a, b| a.figure_id.cmp(&b.figure_id));

    let mut skylanders_states: Vec<SkylanderState> =
        app.skylanders_states.values().cloned().collect();
    skylanders_states.sort_by(|a, b| a.skylander_id.cmp(&b.skylander_id));

    let export = ExportData {
        version: "1.0".to_string(),
        export_date: chrono::Utc::now().to_rfc3339(),
        console_states,
        consoles,
        lego_dimensions_states,
        skylanders_states,
    };

    serde_json::to_string_pretty(&export)
}

#[cfg(target_arch = "wasm32")]
fn download_json_web(json: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let window = web_sys::window().ok_or_else(|| web_error("No browser window is available"))?;
    let document = window
        .document()
        .ok_or_else(|| web_error("No browser document is available"))?;
    let body = document
        .body()
        .ok_or_else(|| web_error("No document body is available"))?;
    let blob =
        web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&js_sys::JsString::from(json)))
            .map_err(|err| web_error(format!("Failed to create export blob: {err:?}")))?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|err| web_error(format!("Failed to create export URL: {err:?}")))?;

    let result = (|| {
        let a = document
            .create_element("a")
            .map_err(|err| web_error(format!("Failed to create download link: {err:?}")))?
            .dyn_into::<web_sys::HtmlElement>()
            .map_err(|err| web_error(format!("Download link is not an HTML element: {err:?}")))?;
        a.set_attribute("href", &url)
            .map_err(|err| web_error(format!("Failed to set download href: {err:?}")))?;
        a.set_attribute("download", filename)
            .map_err(|err| web_error(format!("Failed to set download filename: {err:?}")))?;
        a.style()
            .set_property("display", "none")
            .map_err(|err| web_error(format!("Failed to hide download link: {err:?}")))?;
        body.append_child(&a)
            .map_err(|err| web_error(format!("Failed to attach download link: {err:?}")))?;
        a.click();
        body.remove_child(&a)
            .map_err(|err| web_error(format!("Failed to remove download link: {err:?}")))?;
        Ok(())
    })();

    let revoke_result = web_sys::Url::revoke_object_url(&url)
        .map_err(|err| web_error(format!("Failed to revoke export URL: {err:?}")));

    result.and(revoke_result)
}

#[cfg(target_arch = "wasm32")]
fn web_error(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        message.into(),
    ))
}

#[cfg(target_arch = "wasm32")]
fn log_web_error(message: &str) {
    use wasm_bindgen::JsValue;

    web_sys::console::error_1(&JsValue::from_str(message));
}
