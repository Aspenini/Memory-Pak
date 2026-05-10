use crate::models::StatusState;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryInput {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub filter_by: Option<String>,
    #[serde(default)]
    pub console_id: Option<String>,
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

pub(crate) fn normalize_for_search(text: &str) -> String {
    text.nfd()
        .filter(|c| {
            let code = *c as u32;
            let is_combining_mark = (0x0300..=0x036F).contains(&code)
                || (0x1AB0..=0x1AFF).contains(&code)
                || (0x1DC0..=0x1DFF).contains(&code)
                || (0x20D0..=0x20FF).contains(&code)
                || (0xFE20..=0xFE2F).contains(&code);

            !is_combining_mark
                && !matches!(*c, '\'' | ':' | '-' | '_' | '.' | ',' | '!' | '?' | ';')
        })
        .collect::<String>()
        .to_lowercase()
}

pub(crate) fn normalized_query(value: Option<&str>) -> Option<String> {
    let query = normalize_for_search(value.unwrap_or_default());
    (!query.trim().is_empty()).then_some(query)
}

pub(crate) fn normalized_filter(value: Option<&str>) -> &str {
    value.unwrap_or("all")
}

pub(crate) fn status_matches(state: &StatusState, filter: &str) -> bool {
    match filter {
        "owned" => state.owned,
        "favorites" | "favorite" => state.favorite,
        "wishlist" => state.wishlist,
        "notOwned" | "not_owned" => !state.owned,
        _ => true,
    }
}

pub(crate) fn status_score(state: &StatusState) -> u8 {
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
