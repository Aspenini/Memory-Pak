use std::collections::HashMap;
use std::sync::OnceLock;

use crate::compiled::{
    CompiledCatalog, CompiledCollectible, CompiledCollection, CompiledConsole, CompiledGame,
};
use crate::ids::EntryId;
use crate::model::{Catalog, Collectible, Collection, Console, Game};

static COMPILED_CATALOG_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/catalog.postcard"));

static CATALOG: OnceLock<Catalog> = OnceLock::new();

pub fn catalog() -> &'static Catalog {
    CATALOG.get_or_init(build_catalog)
}

fn build_catalog() -> Catalog {
    let compiled: CompiledCatalog =
        postcard::from_bytes(COMPILED_CATALOG_BYTES).expect("compiled catalog should deserialize");

    let consoles = compiled.consoles.into_iter().map(into_console).collect();

    let mut games: HashMap<EntryId, Game> = HashMap::with_capacity(compiled.games.len());
    for game in compiled.games {
        let game = into_game(game);
        games.insert(game.id.clone(), game);
    }

    let collections = compiled
        .collections
        .into_iter()
        .map(into_collection)
        .collect();
    let collectibles = compiled
        .collectibles
        .into_iter()
        .map(into_collectible)
        .collect();

    Catalog {
        consoles,
        games,
        collections,
        collectibles,
    }
}

fn into_console(c: CompiledConsole) -> Console {
    Console {
        id: EntryId::from_raw(c.id),
        short_id: c.short_id,
        name: c.name,
        manufacturer: c.manufacturer,
        family: c.family,
        form_factor: c.form_factor,
        generation: c.generation,
        abbreviation: c.abbreviation,
    }
}

fn into_game(g: CompiledGame) -> Game {
    Game {
        id: EntryId::from_raw(g.id),
        console_id: EntryId::from_raw(g.console_id),
        console_short_id: g.console_short_id,
        title: g.title,
        developer: g.developer,
        publisher: g.publisher,
        year: g.year,
    }
}

fn into_collection(c: CompiledCollection) -> Collection {
    Collection {
        id: c.id,
        name: c.name,
        manufacturer: c.manufacturer,
        kind: c.kind,
    }
}

fn into_collectible(c: CompiledCollectible) -> Collectible {
    Collectible {
        id: EntryId::from_raw(c.id),
        collection_id: c.collection_id,
        name: c.name,
        category: c.category,
        group: c.group,
        variant: c.variant,
        year: c.year,
    }
}
