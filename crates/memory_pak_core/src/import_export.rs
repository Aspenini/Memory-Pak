use serde::{Deserialize, Serialize};

use crate::ids::EntryId;
use crate::model::{EntryState, PersistedState};

pub const EXPORT_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportEntry {
    pub id: EntryId,
    #[serde(default)]
    pub owned: bool,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub wishlist: bool,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportData {
    pub version: String,
    pub exported_at: String,
    pub entries: Vec<ExportEntry>,
}

pub fn export_json_from_state(state: &PersistedState) -> Result<String, serde_json::Error> {
    let mut entries: Vec<ExportEntry> = state
        .entries
        .iter()
        .filter(|(_, state)| !state.is_empty())
        .map(|(id, state)| ExportEntry {
            id: id.clone(),
            owned: state.owned,
            favorite: state.favorite,
            wishlist: state.wishlist,
            notes: state.notes.clone(),
        })
        .collect();

    entries.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));

    let export = ExportData {
        version: EXPORT_VERSION.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        entries,
    };

    serde_json::to_string_pretty(&export)
}

pub fn apply_import(state: &mut PersistedState, import: ExportData) {
    for entry in import.entries {
        state.entries.insert(
            entry.id,
            EntryState {
                owned: entry.owned,
                favorite: entry.favorite,
                wishlist: entry.wishlist,
                notes: entry.notes,
            },
        );
    }
}
