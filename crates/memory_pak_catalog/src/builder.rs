use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::{
    encode, normalize_for_search, CatalogArchive, CollectibleRecord, CollectionRecord,
    ConsoleRecord, FacetPosting, GameRecord, KindIndex, PartialDate, PostingTerm, RegionalReleases,
    SearchIndex, SortOrders, StringId,
};

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("unable to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{path}: {message}")]
    Validation { path: PathBuf, message: String },
    #[error("unable to write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub bytes: Vec<u8>,
    pub source_digest: [u8; 32],
    pub console_count: usize,
    pub game_count: usize,
    pub collectible_count: usize,
    pub source_files: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ConsolesFile {
    schema_version: u16,
    consoles: Vec<RawConsole>,
}

#[derive(Debug, Deserialize)]
struct RawConsole {
    id: String,
    name: String,
    manufacturer: String,
    #[serde(default)]
    family: String,
    #[serde(default)]
    form_factor: String,
    generation: Option<u16>,
    #[serde(default)]
    abbreviation: String,
    launch_year: Option<u16>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GameFile {
    schema_version: u16,
    console: RawGameConsole,
    counts: Option<RawCounts>,
    #[serde(default)]
    games: Vec<RawGame>,
}

#[derive(Debug, Deserialize)]
struct RawGameConsole {
    id: String,
}

#[derive(Debug, Deserialize)]
struct RawCounts {
    total: usize,
}

#[derive(Debug, Deserialize)]
struct RawGame {
    title: String,
    slug: String,
    #[serde(default)]
    developer: String,
    #[serde(default)]
    publisher: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    category: String,
    first_release: Option<String>,
    #[serde(default)]
    releases: RawReleases,
}

#[derive(Debug, Deserialize, Default)]
struct RawReleases {
    jp: Option<String>,
    na: Option<String>,
    pal: Option<String>,
    eu: Option<String>,
    au: Option<String>,
    br: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CollectibleFile {
    schema_version: u16,
    collection: RawCollection,
    counts: Option<RawCounts>,
    #[serde(default)]
    items: Vec<RawCollectible>,
}

#[derive(Debug, Deserialize)]
struct RawCollection {
    id: String,
    name: String,
    #[serde(default)]
    manufacturer: String,
    #[serde(default, rename = "type")]
    kind: String,
}

#[derive(Debug, Deserialize)]
struct RawCollectible {
    name: String,
    slug: String,
    #[serde(default)]
    category: String,
    pack: Option<String>,
    game: Option<String>,
    base_color: Option<String>,
    year: Option<u16>,
}

#[derive(Default)]
struct Interner {
    by_value: BTreeMap<String, StringId>,
    strings: Vec<String>,
}

impl Interner {
    fn new() -> Self {
        let mut value = Self::default();
        value.intern("");
        value
    }

    fn intern(&mut self, value: impl AsRef<str>) -> StringId {
        let value = value.as_ref();
        if let Some(id) = self.by_value.get(value) {
            return *id;
        }
        let id = self.strings.len() as StringId;
        self.strings.push(value.to_string());
        self.by_value.insert(value.to_string(), id);
        id
    }
}

struct PendingConsole {
    id: String,
    short_id: String,
    name: String,
    manufacturer: String,
    family: String,
    form_factor: String,
    generation: u16,
    abbreviation: String,
    launch_year: u16,
    aliases: Vec<String>,
    search_text: String,
}

struct PendingGame {
    id: String,
    console_id: String,
    title: String,
    developer: String,
    publisher: String,
    status: String,
    category: String,
    first_release: Option<PartialDate>,
    releases: RegionalReleases,
    year: u16,
    search_text: String,
}

struct PendingCollection {
    id: String,
    name: String,
    manufacturer: String,
    kind: String,
}

struct PendingCollectible {
    id: String,
    collection_id: String,
    name: String,
    category: String,
    group: String,
    variant: String,
    year: u16,
    search_text: String,
}

pub fn compile_database(database_dir: &Path) -> Result<BuildOutput, BuildError> {
    let consoles_path = database_dir.join("consoles.json");
    let game_paths = json_files(&database_dir.join("games"))?;
    let collectible_paths = json_files(&database_dir.join("collectibles"))?;
    let mut source_files = vec![consoles_path.clone()];
    source_files.extend(game_paths.iter().cloned());
    source_files.extend(collectible_paths.iter().cloned());

    let mut source_hasher = blake3::Hasher::new();
    for path in &source_files {
        let relative = path.strip_prefix(database_dir).unwrap_or(path);
        source_hasher.update(relative.to_string_lossy().as_bytes());
        source_hasher.update(&[0]);
        source_hasher.update(&read(path)?);
        source_hasher.update(&[0xff]);
    }
    let source_digest = *source_hasher.finalize().as_bytes();

    let consoles_file: ConsolesFile = parse_json(&consoles_path)?;
    validate_schema(consoles_file.schema_version, &consoles_path)?;
    let mut console_ids = BTreeSet::new();
    let mut consoles = Vec::with_capacity(consoles_file.consoles.len());
    for raw in consoles_file.consoles {
        require_id(&raw.id, &consoles_path, "console")?;
        require_text(&raw.name, &consoles_path, "console name")?;
        if !console_ids.insert(raw.id.clone()) {
            return validation(&consoles_path, format!("duplicate console id {:?}", raw.id));
        }
        let search_text = normalize_for_search(
            &[
                raw.name.as_str(),
                raw.manufacturer.as_str(),
                raw.abbreviation.as_str(),
            ]
            .into_iter()
            .chain(raw.aliases.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" "),
        );
        consoles.push(PendingConsole {
            id: format!("console:{}", raw.id),
            short_id: raw.id,
            name: raw.name,
            manufacturer: raw.manufacturer,
            family: raw.family,
            form_factor: raw.form_factor,
            generation: raw.generation.unwrap_or(0),
            abbreviation: raw.abbreviation,
            launch_year: raw.launch_year.unwrap_or(0),
            aliases: raw.aliases,
            search_text,
        });
    }
    consoles.sort_by(|a, b| a.id.cmp(&b.id));

    let console_search: HashMap<&str, (&str, &str, &str, &[String])> = consoles
        .iter()
        .map(|c| {
            (
                c.short_id.as_str(),
                (
                    c.name.as_str(),
                    c.manufacturer.as_str(),
                    c.abbreviation.as_str(),
                    c.aliases.as_slice(),
                ),
            )
        })
        .collect();

    let mut game_ids = BTreeSet::new();
    let mut games = Vec::new();
    for path in game_paths {
        let parsed: GameFile = parse_json(&path)?;
        validate_schema(parsed.schema_version, &path)?;
        if !console_ids.contains(&parsed.console.id) {
            return validation(&path, format!("unknown console id {:?}", parsed.console.id));
        }
        if let Some(counts) = parsed.counts {
            if counts.total != parsed.games.len() {
                return validation(
                    &path,
                    format!(
                        "declared total {} does not match {} games",
                        counts.total,
                        parsed.games.len()
                    ),
                );
            }
        }
        let console_meta = console_search[parsed.console.id.as_str()];
        for raw in parsed.games {
            require_text(&raw.title, &path, "game title")?;
            require_id(&raw.slug, &path, "game slug")?;
            let id = format!("game:{}/{}", parsed.console.id, raw.slug);
            if !game_ids.insert(id.clone()) {
                return validation(&path, format!("duplicate stable id {id:?}"));
            }
            let first_release = parse_optional_date(raw.first_release.as_deref(), &path, &id)?;
            let releases = RegionalReleases {
                jp: parse_optional_date(raw.releases.jp.as_deref(), &path, &id)?,
                na: parse_optional_date(raw.releases.na.as_deref(), &path, &id)?,
                pal: parse_optional_date(raw.releases.pal.as_deref(), &path, &id)?,
                eu: parse_optional_date(raw.releases.eu.as_deref(), &path, &id)?,
                au: parse_optional_date(raw.releases.au.as_deref(), &path, &id)?,
                br: parse_optional_date(raw.releases.br.as_deref(), &path, &id)?,
            };
            let console_aliases = console_meta.3.join(" ");
            let search_text = normalize_for_search(&format!(
                "{} {} {} {} {} {} {}",
                raw.title,
                raw.developer,
                raw.publisher,
                console_meta.0,
                console_meta.1,
                console_meta.2,
                console_aliases
            ));
            games.push(PendingGame {
                id,
                console_id: parsed.console.id.clone(),
                title: raw.title,
                developer: raw.developer,
                publisher: raw.publisher,
                status: raw.status,
                category: raw.category,
                year: first_release.map(|date| date.year).unwrap_or(0),
                first_release,
                releases,
                search_text,
            });
        }
    }
    games.sort_by(|a, b| a.id.cmp(&b.id));

    let mut collection_ids = BTreeSet::new();
    let mut collectible_ids = BTreeSet::new();
    let mut collections = Vec::new();
    let mut collectibles = Vec::new();
    for path in collectible_paths {
        let parsed: CollectibleFile = parse_json(&path)?;
        validate_schema(parsed.schema_version, &path)?;
        require_id(&parsed.collection.id, &path, "collection id")?;
        require_text(&parsed.collection.name, &path, "collection name")?;
        if !collection_ids.insert(parsed.collection.id.clone()) {
            return validation(
                &path,
                format!("duplicate collection id {:?}", parsed.collection.id),
            );
        }
        if let Some(counts) = parsed.counts {
            if counts.total != parsed.items.len() {
                return validation(
                    &path,
                    format!(
                        "declared total {} does not match {} collectibles",
                        counts.total,
                        parsed.items.len()
                    ),
                );
            }
        }
        let collection_id = parsed.collection.id.clone();
        let collection_name = parsed.collection.name.clone();
        collections.push(PendingCollection {
            id: collection_id.clone(),
            name: parsed.collection.name,
            manufacturer: parsed.collection.manufacturer,
            kind: parsed.collection.kind,
        });
        for raw in parsed.items {
            require_text(&raw.name, &path, "collectible name")?;
            require_id(&raw.slug, &path, "collectible slug")?;
            let id = format!("collectible:{collection_id}/{}", raw.slug);
            if !collectible_ids.insert(id.clone()) {
                return validation(
                    &path,
                    format!(
                        "duplicate stable id {id:?}; give each source item a stable unique slug"
                    ),
                );
            }
            let group = raw.pack.or(raw.game).unwrap_or_default();
            let variant = raw.base_color.unwrap_or_default();
            let search_text = normalize_for_search(&format!(
                "{} {} {} {} {}",
                raw.name, collection_name, raw.category, group, variant
            ));
            collectibles.push(PendingCollectible {
                id,
                collection_id: collection_id.clone(),
                name: raw.name,
                category: raw.category,
                group,
                variant,
                year: raw.year.unwrap_or(0),
                search_text,
            });
        }
    }
    collections.sort_by(|a, b| a.id.cmp(&b.id));
    collectibles.sort_by(|a, b| a.id.cmp(&b.id));

    let mut interner = Interner::new();
    let console_ordinals: HashMap<&str, u32> = consoles
        .iter()
        .enumerate()
        .map(|(index, c)| (c.short_id.as_str(), index as u32))
        .collect();
    let collection_ordinals: HashMap<&str, u32> = collections
        .iter()
        .enumerate()
        .map(|(index, c)| (c.id.as_str(), index as u32))
        .collect();

    let console_records: Vec<ConsoleRecord> = consoles
        .iter()
        .map(|c| ConsoleRecord {
            id: interner.intern(&c.id),
            short_id: interner.intern(&c.short_id),
            name: interner.intern(&c.name),
            manufacturer: interner.intern(&c.manufacturer),
            family: interner.intern(&c.family),
            form_factor: interner.intern(&c.form_factor),
            generation: c.generation,
            abbreviation: interner.intern(&c.abbreviation),
            launch_year: c.launch_year,
            aliases: c.aliases.iter().map(|v| interner.intern(v)).collect(),
            search_text: interner.intern(&c.search_text),
        })
        .collect();
    let game_records: Vec<GameRecord> = games
        .iter()
        .map(|g| GameRecord {
            id: interner.intern(&g.id),
            console: console_ordinals[g.console_id.as_str()],
            title: interner.intern(&g.title),
            developer: interner.intern(&g.developer),
            publisher: interner.intern(&g.publisher),
            status: interner.intern(&g.status),
            category: interner.intern(&g.category),
            first_release: g.first_release,
            releases: g.releases.clone(),
            year: g.year,
            search_text: interner.intern(&g.search_text),
        })
        .collect();
    let collection_records: Vec<CollectionRecord> = collections
        .iter()
        .map(|c| CollectionRecord {
            id: interner.intern(&c.id),
            name: interner.intern(&c.name),
            manufacturer: interner.intern(&c.manufacturer),
            kind: interner.intern(&c.kind),
        })
        .collect();
    let collectible_records: Vec<CollectibleRecord> = collectibles
        .iter()
        .map(|c| CollectibleRecord {
            id: interner.intern(&c.id),
            collection: collection_ordinals[c.collection_id.as_str()],
            name: interner.intern(&c.name),
            category: interner.intern(&c.category),
            group: interner.intern(&c.group),
            variant: interner.intern(&c.variant),
            year: c.year,
            search_text: interner.intern(&c.search_text),
        })
        .collect();

    let console_orders = SortOrders {
        name: sorted_ordinals(consoles.len(), |a, b| {
            consoles[a].name.cmp(&consoles[b].name)
        }),
        manufacturer: sorted_ordinals(consoles.len(), |a, b| {
            consoles[a]
                .manufacturer
                .cmp(&consoles[b].manufacturer)
                .then_with(|| consoles[a].name.cmp(&consoles[b].name))
        }),
        ..Default::default()
    };
    let console_index = KindIndex {
        search: build_search_index(
            consoles.iter().map(|c| c.search_text.as_str()),
            &mut interner,
        ),
        facets: vec![],
        ranks: rank_orders(&console_orders),
        orders: console_orders,
    };
    let game_orders = SortOrders {
        name: sorted_ordinals(games.len(), |a, b| games[a].title.cmp(&games[b].title)),
        year: sorted_ordinals(games.len(), |a, b| {
            games[a]
                .year
                .cmp(&games[b].year)
                .then_with(|| games[a].title.cmp(&games[b].title))
        }),
        ..Default::default()
    };
    let game_index = KindIndex {
        search: build_search_index(games.iter().map(|g| g.search_text.as_str()), &mut interner),
        facets: build_facets(
            games
                .iter()
                .enumerate()
                .map(|(ordinal, game)| (format!("console:{}", game.console_id), ordinal as u32)),
            &mut interner,
        ),
        ranks: rank_orders(&game_orders),
        orders: game_orders,
    };
    let collectible_orders = SortOrders {
        name: sorted_ordinals(collectibles.len(), |a, b| {
            collectibles[a].name.cmp(&collectibles[b].name)
        }),
        collection: sorted_ordinals(collectibles.len(), |a, b| {
            collectibles[a]
                .collection_id
                .cmp(&collectibles[b].collection_id)
                .then_with(|| collectibles[a].name.cmp(&collectibles[b].name))
        }),
        category: sorted_ordinals(collectibles.len(), |a, b| {
            collectibles[a]
                .category
                .cmp(&collectibles[b].category)
                .then_with(|| collectibles[a].name.cmp(&collectibles[b].name))
        }),
        group: sorted_ordinals(collectibles.len(), |a, b| {
            collectibles[a]
                .group
                .cmp(&collectibles[b].group)
                .then_with(|| collectibles[a].name.cmp(&collectibles[b].name))
        }),
        variant: sorted_ordinals(collectibles.len(), |a, b| {
            collectibles[a]
                .variant
                .cmp(&collectibles[b].variant)
                .then_with(|| collectibles[a].name.cmp(&collectibles[b].name))
        }),
        year: sorted_ordinals(collectibles.len(), |a, b| {
            collectibles[a]
                .year
                .cmp(&collectibles[b].year)
                .then_with(|| collectibles[a].name.cmp(&collectibles[b].name))
        }),
        ..Default::default()
    };
    let collectible_index = KindIndex {
        search: build_search_index(
            collectibles.iter().map(|c| c.search_text.as_str()),
            &mut interner,
        ),
        facets: build_facets(
            collectibles
                .iter()
                .enumerate()
                .map(|(ordinal, item)| (item.collection_id.clone(), ordinal as u32)),
            &mut interner,
        ),
        ranks: rank_orders(&collectible_orders),
        orders: collectible_orders,
    };

    let archive = CatalogArchive {
        strings: interner.strings,
        consoles: console_records,
        games: game_records,
        collections: collection_records,
        collectibles: collectible_records,
        console_index,
        game_index,
        collectible_index,
    };
    let output = BuildOutput {
        bytes: encode(&archive, source_digest),
        source_digest,
        console_count: archive.consoles.len(),
        game_count: archive.games.len(),
        collectible_count: archive.collectibles.len(),
        source_files,
    };
    Ok(output)
}

pub fn compile_to_path(database_dir: &Path, destination: &Path) -> Result<BuildOutput, BuildError> {
    let output = compile_database(database_dir)?;
    fs::write(destination, &output.bytes).map_err(|source| BuildError::Write {
        path: destination.to_path_buf(),
        source,
    })?;
    Ok(output)
}

fn build_search_index<'a>(
    values: impl Iterator<Item = &'a str>,
    interner: &mut Interner,
) -> SearchIndex {
    let mut by_term: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for (ordinal, value) in values.enumerate() {
        let chars: Vec<char> = value.chars().collect();
        let mut record_terms = BTreeSet::new();
        for width in 1..=3 {
            for part in chars.windows(width) {
                record_terms.insert(part.iter().collect::<String>());
            }
        }
        for term in record_terms {
            by_term.entry(term).or_default().push(ordinal as u32);
        }
    }
    let mut terms = Vec::with_capacity(by_term.len());
    let mut postings = Vec::new();
    for (term, ordinals) in by_term {
        let start = postings.len() as u32;
        let mut previous = 0;
        for (index, ordinal) in ordinals.iter().copied().enumerate() {
            let delta = if index == 0 {
                ordinal
            } else {
                ordinal - previous
            };
            postings.push(delta);
            previous = ordinal;
        }
        terms.push(PostingTerm {
            term: interner.intern(term),
            start,
            len: ordinals.len() as u32,
        });
    }
    SearchIndex { terms, postings }
}

fn build_facets(
    values: impl Iterator<Item = (String, u32)>,
    interner: &mut Interner,
) -> Vec<FacetPosting> {
    let mut facets: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for (value, ordinal) in values {
        facets.entry(value).or_default().push(ordinal);
    }
    facets
        .into_iter()
        .map(|(value, ordinals)| FacetPosting {
            value: interner.intern(value),
            ordinals,
        })
        .collect()
}

fn sorted_ordinals(
    len: usize,
    mut compare: impl FnMut(usize, usize) -> std::cmp::Ordering,
) -> Vec<u32> {
    let mut values: Vec<u32> = (0..len as u32).collect();
    values.sort_by(|a, b| compare(*a as usize, *b as usize));
    values
}

fn rank_orders(orders: &SortOrders) -> SortOrders {
    SortOrders {
        name: ordinal_ranks(&orders.name),
        manufacturer: ordinal_ranks(&orders.manufacturer),
        year: ordinal_ranks(&orders.year),
        collection: ordinal_ranks(&orders.collection),
        category: ordinal_ranks(&orders.category),
        group: ordinal_ranks(&orders.group),
        variant: ordinal_ranks(&orders.variant),
    }
}

fn ordinal_ranks(order: &[u32]) -> Vec<u32> {
    let mut ranks = vec![0; order.len()];
    for (rank, ordinal) in order.iter().copied().enumerate() {
        ranks[ordinal as usize] = rank as u32;
    }
    ranks
}

fn parse_optional_date(
    value: Option<&str>,
    path: &Path,
    id: &str,
) -> Result<Option<PartialDate>, BuildError> {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    parse_date(value)
        .map(Some)
        .map_err(|message| BuildError::Validation {
            path: path.to_path_buf(),
            message: format!("{id}: invalid date {value:?}: {message}"),
        })
}

fn parse_date(value: &str) -> Result<PartialDate, String> {
    let parts: Vec<&str> = value.split('-').collect();
    if !(1..=3).contains(&parts.len()) {
        return Err("expected YYYY, YYYY-MM, or YYYY-MM-DD".into());
    }
    let year: u16 = parts[0].parse().map_err(|_| "invalid year")?;
    if !(1900..=2200).contains(&year) {
        return Err("year is outside 1900..=2200".into());
    }
    let month = if parts.len() >= 2 {
        parts[1].parse::<u8>().map_err(|_| "invalid month")?
    } else {
        0
    };
    let day = if parts.len() == 3 {
        parts[2].parse::<u8>().map_err(|_| "invalid day")?
    } else {
        0
    };
    if month > 12 || (parts.len() >= 2 && month == 0) {
        return Err("month is outside 1..=12".into());
    }
    if day > 31 || (parts.len() == 3 && day == 0) {
        return Err("day is outside 1..=31".into());
    }
    Ok(PartialDate { year, month, day })
}

fn validate_schema(version: u16, path: &Path) -> Result<(), BuildError> {
    if version != 1 {
        return validation(path, format!("unsupported source schema version {version}"));
    }
    Ok(())
}

fn require_text(value: &str, path: &Path, field: &str) -> Result<(), BuildError> {
    if value.trim().is_empty() {
        return validation(path, format!("{field} must not be empty"));
    }
    Ok(())
}

fn require_id(value: &str, path: &Path, field: &str) -> Result<(), BuildError> {
    if value.is_empty()
        || !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '~'))
    {
        return validation(
            path,
            format!("{field} {value:?} must use lowercase ASCII letters, digits, '-' or '~'"),
        );
    }
    Ok(())
}

fn json_files(dir: &Path) -> Result<Vec<PathBuf>, BuildError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(|source| BuildError::Read {
        path: dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| BuildError::Read {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn read(path: &Path) -> Result<Vec<u8>, BuildError> {
    fs::read(path).map_err(|source| BuildError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, BuildError> {
    serde_json::from_slice(&read(path)?).map_err(|source| BuildError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn validation<T>(path: &Path, message: String) -> Result<T, BuildError> {
    Err(BuildError::Validation {
        path: path.to_path_buf(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(game_json: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "memory-pak-catalog-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("games")).unwrap();
        fs::create_dir_all(root.join("collectibles")).unwrap();
        fs::write(
            root.join("consoles.json"),
            r#"{
              "schema_version": 1,
              "consoles": [{
                "id": "test", "name": "Test Console", "manufacturer": "Test",
                "family": "", "form_factor": "home", "generation": 1,
                "abbreviation": "TEST", "launch_year": 2000, "aliases": []
              }]
            }"#,
        )
        .unwrap();
        fs::write(root.join("games/test.json"), game_json).unwrap();
        root
    }

    #[test]
    fn parses_partial_dates() {
        assert_eq!(
            parse_date("1994").unwrap(),
            PartialDate {
                year: 1994,
                month: 0,
                day: 0
            }
        );
        assert_eq!(
            parse_date("1994-12-03").unwrap(),
            PartialDate {
                year: 1994,
                month: 12,
                day: 3
            }
        );
        assert!(parse_date("1994-99").is_err());
    }

    #[test]
    fn full_catalog_build_is_byte_for_byte_deterministic() {
        let database = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../database");
        let first = compile_database(&database).unwrap();
        let second = compile_database(&database).unwrap();
        assert_eq!(first.source_digest, second.source_digest);
        assert_eq!(first.bytes, second.bytes);
    }

    #[test]
    fn rejects_declared_count_mismatches() {
        let root = fixture(
            r#"{
              "schema_version": 1,
              "console": {"id": "test"},
              "counts": {"total": 2},
              "games": [{
                "title": "One", "slug": "one", "first_release": "2000",
                "releases": {}
              }]
            }"#,
        );
        let error = compile_database(&root).unwrap_err().to_string();
        assert!(error.contains("declared total"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_dates_and_duplicate_stable_ids() {
        let invalid_date = fixture(
            r#"{
              "schema_version": 1,
              "console": {"id": "test"},
              "counts": {"total": 1},
              "games": [{
                "title": "One", "slug": "one", "first_release": "2000-99",
                "releases": {}
              }]
            }"#,
        );
        assert!(compile_database(&invalid_date)
            .unwrap_err()
            .to_string()
            .contains("invalid date"));
        fs::remove_dir_all(invalid_date).unwrap();

        let duplicate = fixture(
            r#"{
              "schema_version": 1,
              "console": {"id": "test"},
              "counts": {"total": 2},
              "games": [
                {"title": "One", "slug": "same", "releases": {}},
                {"title": "Two", "slug": "same", "releases": {}}
              ]
            }"#,
        );
        assert!(compile_database(&duplicate)
            .unwrap_err()
            .to_string()
            .contains("duplicate stable id"));
        fs::remove_dir_all(duplicate).unwrap();
    }

    #[test]
    fn rejects_unknown_relationships_and_source_schema_versions() {
        let unknown_console = fixture(
            r#"{
              "schema_version": 1,
              "console": {"id": "missing"},
              "counts": {"total": 0},
              "games": []
            }"#,
        );
        assert!(compile_database(&unknown_console)
            .unwrap_err()
            .to_string()
            .contains("unknown console id"));
        fs::remove_dir_all(unknown_console).unwrap();

        let bad_schema = fixture(
            r#"{
              "schema_version": 9,
              "console": {"id": "test"},
              "counts": {"total": 0},
              "games": []
            }"#,
        );
        assert!(compile_database(&bad_schema)
            .unwrap_err()
            .to_string()
            .contains("unsupported source schema"));
        fs::remove_dir_all(bad_schema).unwrap();
    }
}
