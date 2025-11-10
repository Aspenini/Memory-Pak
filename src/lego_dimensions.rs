use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegoDimensionFigure {
    pub name: String,
    pub category: String,
    pub year: u8,
    pub pack_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegoDimensionState {
    pub figure_id: String,
    pub owned: bool,
    pub favorite: bool,
    pub wishlist: bool,
    pub notes: String,
}

pub fn load_lego_dimensions_figures() -> Vec<LegoDimensionFigure> {
    let data = include_str!("../database/legodimensions.json");
    serde_json::from_str::<Vec<LegoDimensionFigure>>(data).unwrap_or_default()
}

pub fn figure_id(figure: &LegoDimensionFigure) -> String {
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
        "lego-{}-{}",
        slugify(&figure.pack_id),
        slugify(&figure.name)
    )
}
