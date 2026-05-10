use directories::ProjectDirs;
use memory_pak_core::{
    get_console_from_id, ConsoleState, GameState, LegoDimensionState, PersistedState,
    SkylanderState,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn load_persisted_state() -> io::Result<PersistedState> {
    let Some(state_dir) = get_state_dir()? else {
        return Ok(PersistedState::default());
    };

    Ok(PersistedState {
        console_states: load_console_states(&state_dir)?,
        game_states: load_game_states(&state_dir)?,
        lego_dimensions_states: load_lego_dimensions_states(&state_dir)?,
        skylanders_states: load_skylanders_states(&state_dir)?,
    })
}

pub fn save_persisted_state(state: &PersistedState) -> io::Result<()> {
    let Some(state_dir) = get_state_dir()? else {
        return Ok(());
    };

    save_console_states(&state_dir, &state.console_states)?;
    save_game_states(&state_dir, &state.game_states)?;
    save_lego_dimensions_states(&state_dir, &state.lego_dimensions_states)?;
    save_skylanders_states(&state_dir, &state.skylanders_states)?;
    Ok(())
}

fn get_state_dir() -> io::Result<Option<PathBuf>> {
    let Some(proj_dirs) = ProjectDirs::from("com", "memorypak", "memory_pak") else {
        return Ok(None);
    };

    let state_dir = proj_dirs.data_dir().join("state");
    fs::create_dir_all(&state_dir)?;
    Ok(Some(state_dir))
}

fn load_console_states(state_dir: &Path) -> io::Result<HashMap<String, ConsoleState>> {
    let path = state_dir.join("consoles.json");
    let states = read_vec_json::<ConsoleState>(&path)?;
    Ok(states
        .into_iter()
        .map(|state| (state.console_id.clone(), state))
        .collect())
}

fn load_game_states(state_dir: &Path) -> io::Result<HashMap<String, GameState>> {
    let mut flat = HashMap::new();

    let entries = match fs::read_dir(state_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(flat),
        Err(err) => return Err(err),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        if matches!(stem, "consoles" | "lego_dimensions" | "skylanders") {
            continue;
        }

        for state in read_vec_json::<GameState>(&path)? {
            flat.insert(state.game_id.clone(), state);
        }
    }

    Ok(flat)
}

fn load_lego_dimensions_states(
    state_dir: &Path,
) -> io::Result<HashMap<String, LegoDimensionState>> {
    let path = state_dir.join("lego_dimensions.json");
    let states = read_vec_json::<LegoDimensionState>(&path)?;
    Ok(states
        .into_iter()
        .map(|state| (state.figure_id.clone(), state))
        .collect())
}

fn load_skylanders_states(state_dir: &Path) -> io::Result<HashMap<String, SkylanderState>> {
    let path = state_dir.join("skylanders.json");
    let states = read_vec_json::<SkylanderState>(&path)?;
    Ok(states
        .into_iter()
        .map(|state| (state.skylander_id.clone(), state))
        .collect())
}

fn save_console_states(state_dir: &Path, states: &HashMap<String, ConsoleState>) -> io::Result<()> {
    let mut values: Vec<ConsoleState> = states.values().cloned().collect();
    values.sort_by(|a, b| a.console_id.cmp(&b.console_id));
    write_vec_json(&state_dir.join("consoles.json"), &values)
}

fn save_game_states(state_dir: &Path, states: &HashMap<String, GameState>) -> io::Result<()> {
    let mut by_console: HashMap<String, Vec<GameState>> = HashMap::new();

    for (game_id, state) in states {
        let console_id = get_console_from_id(game_id);
        if console_id.is_empty() {
            continue;
        }
        by_console
            .entry(console_id.to_string())
            .or_default()
            .push(state.clone());
    }

    for (console_id, mut values) in by_console {
        values.sort_by(|a, b| a.game_id.cmp(&b.game_id));
        write_vec_json(&state_dir.join(format!("{console_id}.json")), &values)?;
    }

    Ok(())
}

fn save_lego_dimensions_states(
    state_dir: &Path,
    states: &HashMap<String, LegoDimensionState>,
) -> io::Result<()> {
    let mut values: Vec<LegoDimensionState> = states.values().cloned().collect();
    values.sort_by(|a, b| a.figure_id.cmp(&b.figure_id));
    write_vec_json(&state_dir.join("lego_dimensions.json"), &values)
}

fn save_skylanders_states(
    state_dir: &Path,
    states: &HashMap<String, SkylanderState>,
) -> io::Result<()> {
    let mut values: Vec<SkylanderState> = states.values().cloned().collect();
    values.sort_by(|a, b| a.skylander_id.cmp(&b.skylander_id));
    write_vec_json(&state_dir.join("skylanders.json"), &values)
}

fn read_vec_json<T>(path: &Path) -> io::Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str::<Vec<T>>(&content)
            .map_err(|err| io::Error::new(ErrorKind::InvalidData, err)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(err),
    }
}

fn write_vec_json<T>(path: &Path, values: &[T]) -> io::Result<()>
where
    T: serde::Serialize,
{
    let json = serde_json::to_string_pretty(values)
        .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;
    fs::write(path, json)
}
