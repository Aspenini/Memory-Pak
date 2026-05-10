mod persistence;

use memory_pak_core::{
    CollectionStats, InitialState, MemoryPakApp, PersistedState, QueryInput, QueryResult,
    SetItemNotesInput, SetItemStatusInput,
};
use persistence::{load_persisted_state, save_persisted_state};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    app: Mutex<MemoryPakApp>,
}

#[tauri::command]
fn load_initial_state(state: State<'_, AppState>) -> Result<InitialState, String> {
    let app = state.app.lock().map_err(lock_error)?;
    Ok(app.initial_state())
}

#[tauri::command]
fn query_consoles(
    input: QueryInput,
    state: State<'_, AppState>,
) -> Result<QueryResult<memory_pak_core::ConsoleView>, String> {
    let app = state.app.lock().map_err(lock_error)?;
    Ok(app.query_consoles(input))
}

#[tauri::command]
fn query_games(
    input: QueryInput,
    state: State<'_, AppState>,
) -> Result<QueryResult<memory_pak_core::GameView>, String> {
    let app = state.app.lock().map_err(lock_error)?;
    Ok(app.query_games(input))
}

#[tauri::command]
fn query_lego(
    input: QueryInput,
    state: State<'_, AppState>,
) -> Result<QueryResult<memory_pak_core::LegoView>, String> {
    let app = state.app.lock().map_err(lock_error)?;
    Ok(app.query_lego(input))
}

#[tauri::command]
fn query_skylanders(
    input: QueryInput,
    state: State<'_, AppState>,
) -> Result<QueryResult<memory_pak_core::SkylanderView>, String> {
    let app = state.app.lock().map_err(lock_error)?;
    Ok(app.query_skylanders(input))
}

#[tauri::command]
fn set_item_status(
    input: SetItemStatusInput,
    state: State<'_, AppState>,
) -> Result<PersistedState, String> {
    let mut app = state.app.lock().map_err(lock_error)?;
    let snapshot = app.set_item_status(input).map_err(|err| err.to_string())?;
    save_persisted_state(&snapshot).map_err(|err| err.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
fn set_item_notes(
    input: SetItemNotesInput,
    state: State<'_, AppState>,
) -> Result<PersistedState, String> {
    let mut app = state.app.lock().map_err(lock_error)?;
    let snapshot = app.set_item_notes(input).map_err(|err| err.to_string())?;
    save_persisted_state(&snapshot).map_err(|err| err.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
fn import_json(json: String, state: State<'_, AppState>) -> Result<PersistedState, String> {
    let mut app = state.app.lock().map_err(lock_error)?;
    let snapshot = app.import_json(&json).map_err(|err| err.to_string())?;
    save_persisted_state(&snapshot).map_err(|err| err.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
fn export_json(state: State<'_, AppState>) -> Result<String, String> {
    let app = state.app.lock().map_err(lock_error)?;
    app.export_json().map_err(|err| err.to_string())
}

#[tauri::command]
fn get_collection_stats(state: State<'_, AppState>) -> Result<CollectionStats, String> {
    let app = state.app.lock().map_err(lock_error)?;
    Ok(app.collection_stats())
}

#[tauri::command]
fn import_from_path(path: String, state: State<'_, AppState>) -> Result<PersistedState, String> {
    let json = std::fs::read_to_string(PathBuf::from(path)).map_err(|err| err.to_string())?;
    import_json(json, state)
}

#[tauri::command]
fn export_to_path(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let json = export_json(state)?;
    std::fs::write(PathBuf::from(path), json).map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = load_persisted_state().unwrap_or_default();
    let app = MemoryPakApp::from_persisted_state(state);

    tauri::Builder::default()
        .manage(AppState {
            app: Mutex::new(app),
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_initial_state,
            query_consoles,
            query_games,
            query_lego,
            query_skylanders,
            set_item_status,
            set_item_notes,
            import_json,
            export_json,
            get_collection_stats,
            import_from_path,
            export_to_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running Memory Pak");
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> String {
    "application state lock was poisoned".to_string()
}
