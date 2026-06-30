use serde::{Deserialize, Serialize};

use crate::ids::normalize_for_search;
use crate::model::ItemKind;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterBy {
    #[default]
    All,
    Owned,
    Favorites,
    Wishlist,
    NotOwned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortKey {
    Title,
    Name,
    Year,
    Status,
    Category,
    Group,
    Collection,
    Variant,
    Manufacturer,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryInput {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub sort_by: Option<SortKey>,
    #[serde(default)]
    pub filter_by: Option<FilterBy>,
    #[serde(default)]
    pub console_id: Option<String>,
    #[serde(default)]
    pub collection_id: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySpec {
    pub kind: ItemKind,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub sort_by: Option<SortKey>,
    #[serde(default)]
    pub filter_by: Option<FilterBy>,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

impl Default for QuerySpec {
    fn default() -> Self {
        Self {
            kind: ItemKind::Console,
            search: None,
            sort_by: None,
            filter_by: None,
            group_id: None,
            offset: None,
            limit: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult<T> {
    pub total: usize,
    pub items: Vec<T>,
}

pub(crate) fn normalized_query(value: Option<&str>) -> Option<String> {
    let raw = value?;
    let normalized = normalize_for_search(raw);
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
