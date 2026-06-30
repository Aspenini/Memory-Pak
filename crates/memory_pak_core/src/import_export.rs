use serde::{Deserialize, Serialize};

use crate::ids::EntryId;
use crate::model::{EntryState, PersistedState, SaveEnvelopeV3, SAVE_SCHEMA_VERSION};

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

pub fn decode_persisted_state(json: &str) -> Result<PersistedState, SaveDecodeError> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let mut state = if value.get("schemaVersion").is_some() {
        let envelope: SaveEnvelopeV3 = serde_json::from_value(value)?;
        if envelope.schema_version != SAVE_SCHEMA_VERSION {
            return Err(SaveDecodeError::UnsupportedSchema(envelope.schema_version));
        }
        PersistedState::from(envelope)
    } else {
        serde_json::from_value(value)?
    };
    state.entries.retain(|_, entry| !entry.is_empty());
    Ok(state)
}

pub fn encode_persisted_state(state: &PersistedState) -> Result<String, serde_json::Error> {
    let envelope = SaveEnvelopeV3 {
        schema_version: SAVE_SCHEMA_VERSION,
        entries: state
            .entries
            .iter()
            .filter(|(_, entry)| !entry.is_empty())
            .map(|(id, entry)| (id.clone(), entry.clone()))
            .collect(),
    };
    serde_json::to_string_pretty(&envelope)
}

#[derive(Debug, thiserror::Error)]
pub enum SaveDecodeError {
    #[error("invalid save JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported save schema {0}")]
    UnsupportedSchema(u32),
}
