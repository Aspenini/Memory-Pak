//! Versioned, deterministic catalog archive used by every Memory Pak target.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

#[cfg(feature = "builder")]
pub mod builder;

pub const MAGIC: &[u8; 8] = b"MPAKCAT\0";
pub const SCHEMA_VERSION: u16 = 1;
const HEADER_LEN: usize = MAGIC.len() + 2 + 32 + 32;

pub type StringId = u32;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PartialDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl PartialDate {
    pub fn precision(self) -> DatePrecision {
        if self.day != 0 {
            DatePrecision::Day
        } else if self.month != 0 {
            DatePrecision::Month
        } else {
            DatePrecision::Year
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RegionalReleases {
    pub jp: Option<PartialDate>,
    pub na: Option<PartialDate>,
    pub pal: Option<PartialDate>,
    pub eu: Option<PartialDate>,
    pub au: Option<PartialDate>,
    pub br: Option<PartialDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleRecord {
    pub id: StringId,
    pub short_id: StringId,
    pub name: StringId,
    pub manufacturer: StringId,
    pub family: StringId,
    pub form_factor: StringId,
    pub generation: u16,
    pub abbreviation: StringId,
    pub launch_year: u16,
    pub aliases: Vec<StringId>,
    pub search_text: StringId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    pub id: StringId,
    pub console: u32,
    pub title: StringId,
    pub developer: StringId,
    pub publisher: StringId,
    pub status: StringId,
    pub category: StringId,
    pub first_release: Option<PartialDate>,
    pub releases: RegionalReleases,
    pub year: u16,
    pub search_text: StringId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub id: StringId,
    pub name: StringId,
    pub manufacturer: StringId,
    pub kind: StringId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectibleRecord {
    pub id: StringId,
    pub collection: u32,
    pub name: StringId,
    pub category: StringId,
    pub group: StringId,
    pub variant: StringId,
    pub year: u16,
    pub search_text: StringId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostingTerm {
    pub term: StringId,
    pub start: u32,
    pub len: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchIndex {
    /// Terms sorted by their resolved string value.
    pub terms: Vec<PostingTerm>,
    /// Delta-encoded dense ordinals for all terms.
    pub postings: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FacetPosting {
    pub value: StringId,
    pub ordinals: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SortOrders {
    pub name: Vec<u32>,
    pub manufacturer: Vec<u32>,
    pub year: Vec<u32>,
    pub collection: Vec<u32>,
    pub category: Vec<u32>,
    pub group: Vec<u32>,
    pub variant: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KindIndex {
    pub search: SearchIndex,
    pub facets: Vec<FacetPosting>,
    pub orders: SortOrders,
    pub ranks: SortOrders,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogArchive {
    pub strings: Vec<String>,
    pub consoles: Vec<ConsoleRecord>,
    pub games: Vec<GameRecord>,
    pub collections: Vec<CollectionRecord>,
    pub collectibles: Vec<CollectibleRecord>,
    pub console_index: KindIndex,
    pub game_index: KindIndex,
    pub collectible_index: KindIndex,
}

impl CatalogArchive {
    pub fn string(&self, id: StringId) -> &str {
        self.strings
            .get(id as usize)
            .map(String::as_str)
            .unwrap_or("")
    }

    pub fn source_counts(&self) -> (usize, usize, usize) {
        (
            self.consoles.len(),
            self.games.len(),
            self.collectibles.len(),
        )
    }
}

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("catalog is truncated")]
    Truncated,
    #[error("catalog magic is invalid")]
    InvalidMagic,
    #[error("unsupported catalog schema {0}")]
    UnsupportedSchema(u16),
    #[error("catalog payload checksum does not match")]
    Checksum,
    #[error("catalog payload is invalid: {0}")]
    Decode(#[from] postcard::Error),
}

#[derive(Debug)]
pub struct DecodedCatalog {
    pub source_digest: [u8; 32],
    pub archive: CatalogArchive,
}

pub fn encode(archive: &CatalogArchive, source_digest: [u8; 32]) -> Vec<u8> {
    let payload = postcard::to_allocvec(archive).expect("catalog archive serialization");
    let payload_digest = *blake3::hash(&payload).as_bytes();
    let mut bytes = Vec::with_capacity(HEADER_LEN + payload.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&SCHEMA_VERSION.to_le_bytes());
    bytes.extend_from_slice(&source_digest);
    bytes.extend_from_slice(&payload_digest);
    bytes.extend_from_slice(&payload);
    bytes
}

pub fn decode(bytes: &[u8]) -> Result<DecodedCatalog, ArchiveError> {
    if bytes.len() < HEADER_LEN {
        return Err(ArchiveError::Truncated);
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return Err(ArchiveError::InvalidMagic);
    }
    let schema_start = MAGIC.len();
    let schema = u16::from_le_bytes([bytes[schema_start], bytes[schema_start + 1]]);
    if schema != SCHEMA_VERSION {
        return Err(ArchiveError::UnsupportedSchema(schema));
    }
    let source_start = schema_start + 2;
    let payload_hash_start = source_start + 32;
    let payload_start = payload_hash_start + 32;

    let mut source_digest = [0; 32];
    source_digest.copy_from_slice(&bytes[source_start..payload_hash_start]);
    let expected_hash = &bytes[payload_hash_start..payload_start];
    let payload = &bytes[payload_start..];
    if blake3::hash(payload).as_bytes() != expected_hash {
        return Err(ArchiveError::Checksum);
    }
    Ok(DecodedCatalog {
        source_digest,
        archive: postcard::from_bytes(payload)?,
    })
}

/// Matches the existing Memory Pak search behavior: case/diacritic folded and
/// punctuation-insensitive while retaining spaces for substring matching.
pub fn normalize_for_search(value: &str) -> String {
    value
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .filter(|c| !matches!(*c, '\'' | ':' | '-' | '_' | '.' | ',' | '!' | '?' | ';'))
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn query_grams(value: &str) -> Vec<String> {
    let chars: Vec<char> = value.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let width = chars.len().min(3);
    let mut grams: Vec<String> = chars
        .windows(width)
        .map(|part| part.iter().collect())
        .collect();
    grams.sort();
    grams.dedup();
    grams
}

fn is_combining_mark(c: char) -> bool {
    let code = c as u32;
    (0x0300..=0x036f).contains(&code)
        || (0x1ab0..=0x1aff).contains(&code)
        || (0x1dc0..=0x1dff).contains(&code)
        || (0x20d0..=0x20ff).contains(&code)
        || (0xfe20..=0xfe2f).contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_normalization_matches_legacy_behavior() {
        assert_eq!(normalize_for_search("Pokémon: Red"), "pokemon red");
        assert_eq!(query_grams("mario"), ["ari", "mar", "rio"]);
        assert_eq!(query_grams("gb"), ["gb"]);
    }

    #[test]
    fn archive_rejects_corruption() {
        let archive = CatalogArchive {
            strings: vec!["".into()],
            consoles: vec![],
            games: vec![],
            collections: vec![],
            collectibles: vec![],
            console_index: KindIndex::default(),
            game_index: KindIndex::default(),
            collectible_index: KindIndex::default(),
        };
        let mut bytes = encode(&archive, [7; 32]);
        assert_eq!(decode(&bytes).unwrap().source_digest, [7; 32]);
        *bytes.last_mut().unwrap() ^= 1;
        assert!(matches!(decode(&bytes), Err(ArchiveError::Checksum)));
    }
}
