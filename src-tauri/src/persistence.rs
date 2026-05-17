use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use memory_pak_core::PersistedState;
use tauri::{AppHandle, Manager, Runtime};

const STATE_FILE: &str = "state.json";

pub fn load_persisted_state<R: Runtime>(app: &AppHandle<R>) -> io::Result<PersistedState> {
    let path = state_path(app)?;

    match read_state_file(&path) {
        Ok(state) => Ok(state),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            load_legacy_state(app).or_else(|legacy_err| {
                if legacy_err.kind() == ErrorKind::NotFound {
                    Ok(PersistedState::default())
                } else {
                    Err(legacy_err)
                }
            })
        }
        Err(err) => Err(err),
    }
}

pub fn save_persisted_state<R: Runtime>(
    app: &AppHandle<R>,
    state: &PersistedState,
) -> io::Result<()> {
    let path = state_path(app)?;

    let json = serde_json::to_string_pretty(state)
        .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;

    write_atomic(&path, json.as_bytes())
}

fn read_state_file(path: &Path) -> io::Result<PersistedState> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str::<PersistedState>(&text)
        .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))
}

fn state_path<R: Runtime>(app: &AppHandle<R>) -> io::Result<PathBuf> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| io::Error::other(err.to_string()))?;
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join(STATE_FILE))
}

fn legacy_state_path() -> io::Result<Option<PathBuf>> {
    let Some(dirs) = ProjectDirs::from("com", "memorypak", "memory_pak") else {
        return Ok(None);
    };
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(Some(data_dir.join(STATE_FILE)))
}

fn load_legacy_state<R: Runtime>(app: &AppHandle<R>) -> io::Result<PersistedState> {
    let Some(legacy_path) = legacy_state_path()? else {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            "legacy data directory is unavailable",
        ));
    };
    let state = read_state_file(&legacy_path)?;
    let _ = save_persisted_state(app, &state);
    Ok(state)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            // Best-effort cleanup on failure.
            let _ = fs::remove_file(&tmp);
            Err(err)
        }
    }
}
