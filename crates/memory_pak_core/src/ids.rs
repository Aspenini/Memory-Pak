use std::fmt;

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryKind {
    Console,
    Game,
    Collectible,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntryKind::Console => "console",
            EntryKind::Game => "game",
            EntryKind::Collectible => "collectible",
        }
    }

    pub fn parse_prefix(value: &str) -> Option<Self> {
        match value {
            "console" => Some(EntryKind::Console),
            "game" => Some(EntryKind::Game),
            "collectible" => Some(EntryKind::Collectible),
            _ => None,
        }
    }
}

/// A persisted entry identifier of the form `kind:locator`.
///
/// Examples:
/// - `console:nes`
/// - `game:nes/super-mario-bros`
/// - `collectible:legodimensions/batman`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntryId(String);

impl EntryId {
    pub fn new(kind: EntryKind, locator: impl AsRef<str>) -> Self {
        Self(format!("{}:{}", kind.as_str(), locator.as_ref()))
    }

    pub fn from_raw(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn kind(&self) -> Option<EntryKind> {
        let (prefix, _) = self.0.split_once(':')?;
        EntryKind::parse_prefix(prefix)
    }

    pub fn locator(&self) -> &str {
        self.0.split_once(':').map(|(_, l)| l).unwrap_or(&self.0)
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for EntryId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for EntryId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Lowercase, Unicode-folded, punctuation-stripped text suitable for substring search.
pub fn normalize_for_search(value: &str) -> String {
    value
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .filter(|c| !matches!(*c, '\'' | ':' | '-' | '_' | '.' | ',' | '!' | '?' | ';'))
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn is_combining_mark(c: char) -> bool {
    let code = c as u32;
    (0x0300..=0x036F).contains(&code)
        || (0x1AB0..=0x1AFF).contains(&code)
        || (0x1DC0..=0x1DFF).contains(&code)
        || (0x20D0..=0x20FF).contains(&code)
        || (0xFE20..=0xFE2F).contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_id_roundtrip() {
        let id = EntryId::new(EntryKind::Game, "nes/super-mario-bros");
        assert_eq!(id.as_str(), "game:nes/super-mario-bros");
        assert_eq!(id.kind(), Some(EntryKind::Game));
        assert_eq!(id.locator(), "nes/super-mario-bros");
    }

    #[test]
    fn search_normalization_strips_punctuation() {
        assert_eq!(normalize_for_search("Pokémon: Red"), "pokemon red");
    }
}
