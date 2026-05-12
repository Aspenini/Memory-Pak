// Shared between build.rs and src/catalog.rs.
//
// Keep this module dependency-free apart from serde so the build script can
// `#[path = "src/compiled.rs"] mod compiled;` it directly.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledCatalog {
    pub consoles: Vec<CompiledConsole>,
    pub games: Vec<CompiledGame>,
    pub collections: Vec<CompiledCollection>,
    pub collectibles: Vec<CompiledCollectible>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledConsole {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub manufacturer: String,
    pub family: String,
    pub form_factor: String,
    pub generation: u32,
    pub abbreviation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledGame {
    pub id: String,
    pub console_id: String,
    pub console_short_id: String,
    pub title: String,
    pub developer: String,
    pub publisher: String,
    pub year: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledCollection {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledCollectible {
    pub id: String,
    pub collection_id: String,
    pub name: String,
    pub category: String,
    pub group: String,
    pub variant: String,
    pub year: u8,
}
