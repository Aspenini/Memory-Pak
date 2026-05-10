use crate::models::{LegoDimensionFigure, Skylander};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn get_console_from_id(game_id: &str) -> &str {
    game_id
        .rsplit_once('-')
        .map(|(console, _)| console)
        .unwrap_or("")
}

pub fn generate_stable_id(
    console_id: &str,
    title: &str,
    publisher: &str,
    release_date: Option<&str>,
) -> String {
    let date = release_date.unwrap_or("");
    format!(
        "{}-{:016x}",
        console_id,
        stable_hash(&[console_id, title, publisher, date])
    )
}

pub fn generate_legacy_id(console_id: &str, title: &str) -> String {
    let mut hasher = DefaultHasher::new();
    console_id.hash(&mut hasher);
    title.hash(&mut hasher);
    format!("{}-{:x}", console_id, hasher.finish())
}

pub fn figure_id(figure: &LegoDimensionFigure) -> String {
    format!(
        "lego-{}-{}",
        slugify(&figure.pack_id),
        slugify(&figure.name)
    )
}

pub fn skylander_id(skylander: &Skylander) -> String {
    format!(
        "skylander-{}-{}-{}",
        slugify(&skylander.game),
        slugify(&skylander.name),
        slugify(&skylander.category)
    )
}

pub(crate) fn parse_year_from_iso_date(date_str: &str) -> u32 {
    date_str
        .split('-')
        .next()
        .and_then(|year| year.parse::<u32>().ok())
        .unwrap_or(0)
}

fn slugify(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn stable_hash(parts: &[&str]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;

    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    hash
}
