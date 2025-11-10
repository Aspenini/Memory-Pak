use crate::Console;
use serde_json;

const CONSOLES_JSON: &str = include_str!("../database/consoles.json");

pub fn get_hardcoded_consoles() -> Vec<Console> {
    match serde_json::from_str::<Vec<Console>>(CONSOLES_JSON) {
        Ok(consoles) => consoles,
        Err(err) => {
            eprintln!("Failed to load consoles.json: {err}");
            Vec::new()
        }
    }
}
