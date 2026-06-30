use std::collections::HashMap;
use std::sync::OnceLock;

use memory_pak_catalog::{decode, CatalogArchive};

use crate::ids::EntryId;
use crate::model::{Catalog, Collectible, Collection, Console, Game};

static COMPILED_CATALOG_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/catalog.bin"));

static CATALOG: OnceLock<Catalog> = OnceLock::new();

pub fn catalog() -> &'static Catalog {
    CATALOG.get_or_init(build_catalog)
}

fn build_catalog() -> Catalog {
    let decoded = decode(COMPILED_CATALOG_BYTES).expect("compiled catalog should validate");
    let archive = decoded.archive;

    let consoles = archive
        .consoles
        .iter()
        .map(|record| Console {
            id: EntryId::from_raw(archive.string(record.id).to_string()),
            short_id: archive.string(record.short_id).to_string(),
            name: archive.string(record.name).to_string(),
            manufacturer: archive.string(record.manufacturer).to_string(),
            family: archive.string(record.family).to_string(),
            form_factor: archive.string(record.form_factor).to_string(),
            generation: u32::from(record.generation),
            abbreviation: archive.string(record.abbreviation).to_string(),
        })
        .collect();

    let mut games = HashMap::with_capacity(archive.games.len());
    for record in &archive.games {
        let console = &archive.consoles[record.console as usize];
        let game = Game {
            id: EntryId::from_raw(archive.string(record.id).to_string()),
            console_id: EntryId::from_raw(archive.string(console.id).to_string()),
            console_short_id: archive.string(console.short_id).to_string(),
            title: archive.string(record.title).to_string(),
            developer: archive.string(record.developer).to_string(),
            publisher: archive.string(record.publisher).to_string(),
            year: u32::from(record.year),
        };
        games.insert(game.id.clone(), game);
    }

    let collections = archive
        .collections
        .iter()
        .map(|record| Collection {
            id: archive.string(record.id).to_string(),
            name: archive.string(record.name).to_string(),
            manufacturer: archive.string(record.manufacturer).to_string(),
            kind: archive.string(record.kind).to_string(),
        })
        .collect();

    let collectibles = archive
        .collectibles
        .iter()
        .map(|record| {
            let collection = &archive.collections[record.collection as usize];
            Collectible {
                id: EntryId::from_raw(archive.string(record.id).to_string()),
                collection_id: archive.string(collection.id).to_string(),
                name: archive.string(record.name).to_string(),
                category: archive.string(record.category).to_string(),
                group: archive.string(record.group).to_string(),
                variant: archive.string(record.variant).to_string(),
                year: u8::try_from(record.year).unwrap_or(u8::MAX),
            }
        })
        .collect();

    Catalog {
        archive,
        source_digest: decoded.source_digest,
        consoles,
        games,
        collections,
        collectibles,
    }
}

pub(crate) fn archive(catalog: &Catalog) -> &CatalogArchive {
    &catalog.archive
}
