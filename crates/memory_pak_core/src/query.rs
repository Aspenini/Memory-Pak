use serde::{Deserialize, Serialize};

use crate::ids::normalize_for_search;
use crate::model::EntryState;

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

pub(crate) fn matches_filter(state: &EntryState, filter: FilterBy) -> bool {
    match filter {
        FilterBy::All => true,
        FilterBy::Owned => state.owned,
        FilterBy::Favorites => state.favorite,
        FilterBy::Wishlist => state.wishlist,
        FilterBy::NotOwned => !state.owned,
    }
}

pub(crate) fn status_score(state: &EntryState) -> u8 {
    if state.owned {
        3
    } else if state.favorite {
        2
    } else if state.wishlist {
        1
    } else {
        0
    }
}

pub(crate) fn paginate<T>(
    items: Vec<T>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> QueryResult<T> {
    let total = items.len();
    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(total - offset);
    let items = items.into_iter().skip(offset).take(limit).collect();
    QueryResult { total, items }
}
