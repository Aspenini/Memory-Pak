mod persistence;

#[cfg(target_os = "android")]
use std::io::{Read, Write};
#[cfg(not(target_os = "android"))]
use std::path::PathBuf;

use memory_pak_core::{
    CollectibleView, CollectionStats, ConsoleView, GameView, InitialState, MemoryPakApp,
    MutationResult, QueryInput, QueryResult, SetItemNotesInput, SetItemStatusInput,
};
use parking_lot::RwLock;
use persistence::{load_persisted_state, save_persisted_state};
use serde::Serialize;
use tauri::{AppHandle, Manager, State, Wry};
#[cfg(target_os = "android")]
use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

struct AppState {
    app: RwLock<MemoryPakApp>,
}

const ANDROID_STORE_URL: &str =
    "https://play.google.com/store/apps/details?id=com.Aspenini.MemoryPak";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AndroidUpdateStatus {
    available: bool,
    version: Option<String>,
    notes: Option<String>,
    can_install_in_app: bool,
    external_url: String,
}

#[tauri::command]
fn load_initial_state(state: State<'_, AppState>) -> InitialState {
    state.app.read().initial_state()
}

#[tauri::command]
fn query_consoles(input: QueryInput, state: State<'_, AppState>) -> QueryResult<ConsoleView> {
    state.app.read().query_consoles(input)
}

#[tauri::command]
fn query_games(input: QueryInput, state: State<'_, AppState>) -> QueryResult<GameView> {
    state.app.read().query_games(input)
}

#[tauri::command]
fn query_collectibles(
    input: QueryInput,
    state: State<'_, AppState>,
) -> QueryResult<CollectibleView> {
    state.app.read().query_collectibles(input)
}

#[tauri::command]
fn set_item_status(
    input: SetItemStatusInput,
    state: State<'_, AppState>,
    app_handle: AppHandle<Wry>,
) -> Result<MutationResult, String> {
    let mut app = state.app.write();
    let result = app.set_item_status(input).map_err(|err| err.to_string())?;
    save_persisted_state(&app_handle, app.persisted_state()).map_err(|err| err.to_string())?;
    Ok(result)
}

#[tauri::command]
fn set_item_notes(
    input: SetItemNotesInput,
    state: State<'_, AppState>,
    app_handle: AppHandle<Wry>,
) -> Result<MutationResult, String> {
    let mut app = state.app.write();
    let result = app.set_item_notes(input).map_err(|err| err.to_string())?;
    save_persisted_state(&app_handle, app.persisted_state()).map_err(|err| err.to_string())?;
    Ok(result)
}

#[tauri::command]
fn import_json(
    json: String,
    state: State<'_, AppState>,
    app_handle: AppHandle<Wry>,
) -> Result<CollectionStats, String> {
    let mut app = state.app.write();
    let stats = app.import_json(&json).map_err(|err| err.to_string())?;
    save_persisted_state(&app_handle, app.persisted_state()).map_err(|err| err.to_string())?;
    Ok(stats)
}

#[tauri::command]
fn export_json(state: State<'_, AppState>) -> Result<String, String> {
    state
        .app
        .read()
        .export_json()
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn get_collection_stats(state: State<'_, AppState>) -> CollectionStats {
    state.app.read().collection_stats()
}

#[tauri::command]
fn import_from_path(
    path: String,
    state: State<'_, AppState>,
    app_handle: AppHandle<Wry>,
) -> Result<CollectionStats, String> {
    let json = read_user_file(&app_handle, &path)?;
    import_json(json, state, app_handle)
}

#[tauri::command]
fn export_to_path(
    path: String,
    state: State<'_, AppState>,
    app_handle: AppHandle<Wry>,
) -> Result<(), String> {
    let json = export_json(state)?;
    write_user_file(&app_handle, &path, json.as_bytes())
}

#[cfg(target_os = "android")]
fn read_user_file(app: &AppHandle<Wry>, path: &str) -> Result<String, String> {
    let file_path = match path.parse::<FilePath>() {
        Ok(file_path) => file_path,
        Err(err) => match err {},
    };
    let mut options = OpenOptions::new();
    options.read(true);
    let mut file = app
        .fs()
        .open(file_path, options)
        .map_err(|err| err.to_string())?;
    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|err| err.to_string())?;
    Ok(json)
}

#[cfg(not(target_os = "android"))]
fn read_user_file(_app: &AppHandle<Wry>, path: &str) -> Result<String, String> {
    std::fs::read_to_string(PathBuf::from(path)).map_err(|err| err.to_string())
}

#[cfg(target_os = "android")]
fn write_user_file(app: &AppHandle<Wry>, path: &str, bytes: &[u8]) -> Result<(), String> {
    let file_path = match path.parse::<FilePath>() {
        Ok(file_path) => file_path,
        Err(err) => match err {},
    };
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    let mut file = app
        .fs()
        .open(file_path, options)
        .map_err(|err| err.to_string())?;
    file.write_all(bytes).map_err(|err| err.to_string())?;
    file.flush().map_err(|err| err.to_string())
}

#[cfg(not(target_os = "android"))]
fn write_user_file(_app: &AppHandle<Wry>, path: &str, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(PathBuf::from(path), bytes).map_err(|err| err.to_string())
}

#[tauri::command]
fn android_check_store_update() -> AndroidUpdateStatus {
    AndroidUpdateStatus {
        available: false,
        version: None,
        notes: Some("Google Play checks for store updates on Android.".to_string()),
        can_install_in_app: false,
        external_url: ANDROID_STORE_URL.to_string(),
    }
}

#[tauri::command]
fn android_start_store_update() -> AndroidUpdateStatus {
    android_check_store_update()
}

#[tauri::command]
fn android_open_update_target() -> String {
    ANDROID_STORE_URL.to_string()
}

#[tauri::command]
fn runtime_platform() -> &'static str {
    #[cfg(target_os = "android")]
    {
        "android"
    }

    #[cfg(not(target_os = "android"))]
    {
        "desktop"
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let state = load_persisted_state(app.handle()).unwrap_or_default();
            app.manage(AppState {
                app: RwLock::new(MemoryPakApp::from_persisted_state(state)),
            });

            #[cfg(desktop)]
            if app
                .config()
                .plugins
                .0
                .get("updater")
                .is_some_and(|config| !config.is_null())
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_initial_state,
            query_consoles,
            query_games,
            query_collectibles,
            set_item_status,
            set_item_notes,
            import_json,
            export_json,
            get_collection_stats,
            import_from_path,
            export_to_path,
            android_check_store_update,
            android_start_store_update,
            android_open_update_target,
            runtime_platform
        ])
        .run(tauri::generate_context!())
        .expect("error while running Memory Pak");
}
