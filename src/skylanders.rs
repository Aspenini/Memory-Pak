use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skylander {
    pub name: String,
    pub game: String,
    pub base_color: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkylanderState {
    pub skylander_id: String,
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

pub fn load_skylanders() -> Vec<Skylander> {
    let data = include_str!("../database/skylanders.json");
    serde_json::from_str::<Vec<Skylander>>(data).unwrap_or_default()
}

pub fn skylander_id(skylander: &Skylander) -> String {
    fn slugify(value: &str) -> String {
        let slug = value
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>();

        slug.split('-')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    format!(
        "skylander-{}-{}-{}",
        slugify(&skylander.game),
        slugify(&skylander.name),
        slugify(&skylander.category)
    )
}

