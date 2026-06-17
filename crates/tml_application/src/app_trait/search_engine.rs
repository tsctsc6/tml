use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::usecase::app::search_music_info::{MusicInfoSearchItem, QueryField};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResults<T> {
    /// Results of the query.
    pub hits: Vec<SearchResult<T>>,
    /// Current page number
    pub page: usize,
    /// Exhaustive number of matches.
    pub total_hits: usize,
    /// Exhaustive number of pages.
    pub total_pages: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult<T> {
    /// The full result.
    #[serde(flatten)]
    pub result: T,

    /// The object that contains information about the matches.
    #[serde(rename = "_matches_position")]
    pub matches_position: HashMap<String, Vec<MatchRange>>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct MatchRange {
    pub start: usize,
    pub length: usize,

    /// If the match is somewhere inside a (potentially nested) array, this
    /// field is set to the index/indices of the matched element(s).
    ///
    /// In the simple case, if the field has the value `["foo", "bar"]`, then
    /// searching for `ba` will return `indices: Some([1])`. If the value
    /// contains multiple nested arrays, the first index describes the most
    /// top-level array, and descending from there. For example, if the value is
    /// `[{ x: "cat" }, "bear", { y: ["dog", "fox"] }]`, searching for `dog`
    /// will return `indices: Some([2, 0])`.
    pub indices: Option<Vec<usize>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[async_trait::async_trait]
pub trait Trait: Send + Sync + Clone + 'static {
    async fn search_music_info(
        &self,
        query: &str,
        hits_per_page: usize,
        page: usize,
        query_field: QueryField,
    ) -> Result<SearchResults<MusicInfoSearchItem>, Error>;
}
