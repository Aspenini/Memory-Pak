use crate::ids::get_console_from_id;
use crate::models::{ConsoleState, GameState, LegoDimensionState, PersistedState, SkylanderState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportData {
    pub version: String,
    pub export_date: String,
    pub console_states: Vec<ConsoleState>,
    pub consoles: Vec<ConsoleExportData>,
    #[serde(default)]
    pub lego_dimensions_states: Vec<LegoDimensionState>,
    #[serde(default)]
    pub skylanders_states: Vec<SkylanderState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsoleExportData {
    pub console_id: String,
    pub games: Vec<GameState>,
}

pub fn export_json_from_state(state: &PersistedState) -> Result<String, serde_json::Error> {
    let mut games_by_console: HashMap<String, Vec<GameState>> = HashMap::new();
    for (game_id, game_state) in &state.game_states {
        let console_id = get_console_from_id(game_id).to_string();
        if console_id.is_empty() {
            continue;
        }
        games_by_console
            .entry(console_id)
            .or_default()
            .push(game_state.clone());
    }

    let mut console_states: Vec<ConsoleState> = state.console_states.values().cloned().collect();
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
        state.lego_dimensions_states.values().cloned().collect();
    lego_dimensions_states.sort_by(|a, b| a.figure_id.cmp(&b.figure_id));

    let mut skylanders_states: Vec<SkylanderState> =
        state.skylanders_states.values().cloned().collect();
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
