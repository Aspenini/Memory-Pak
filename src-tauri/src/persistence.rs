use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use memory_pak_core::PersistedState;

const STATE_FILE: &str = "state.json";

pub fn load_persisted_state() -> io::Result<PersistedState> {
    let Some(path) = state_path()? else {
        return Ok(PersistedState::default());
    };

    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str::<PersistedState>(&text)
            .map_err(|err| io::Error::new(ErrorKind::InvalidData, err)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(PersistedState::default()),
        Err(err) => Err(err),
    }
}

pub fn save_persisted_state(state: &PersistedState) -> io::Result<()> {
    let Some(path) = state_path()? else {
        return Ok(());
    };

    let json = serde_json::to_string_pretty(state)
        .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;

    write_atomic(&path, json.as_bytes())
}

fn state_path() -> io::Result<Option<PathBuf>> {
    let Some(dirs) = ProjectDirs::from("com", "memorypak", "memory_pak") else {
        return Ok(None);
    };
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(Some(data_dir.join(STATE_FILE)))
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
