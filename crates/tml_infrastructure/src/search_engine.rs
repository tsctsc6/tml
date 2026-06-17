use meilisearch_sdk::{client::Client, search::SearchResults};
use tml_application::{
    app_trait::search_engine, usecase::app::search_music_info::MusicInfoSearchItem,
};

#[derive(Clone)]
pub struct SearchEngine {
    client: Client,
    meilisearch_index_name: String,
}

impl SearchEngine {
    pub fn new(client: Client, meilisearch_index_name: &str) -> SearchEngine {
        SearchEngine {
            client,
            meilisearch_index_name: meilisearch_index_name.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl search_engine::Trait for SearchEngine {
    async fn search_music_info(
        &self,
        query: &str,
        hits_per_page: usize,
        page: usize,
    ) -> Result<search_engine::SearchResults<MusicInfoSearchItem>, search_engine::Error> {
        let results: SearchResults<MusicInfoSearchItem> = self
            .client
            .index(self.meilisearch_index_name.clone())
            .search()
            .with_query(query)
            .with_show_matches_position(true)
            .with_hits_per_page(hits_per_page)
            .with_page(page)
            .execute()
            .await
            .map_err(|e| -> search_engine::Error {
                search_engine::Error::Unknown(e.to_string())
            })?;

        // Because use
        // `.with_show_matches_position(true).with_hits_per_page()`
        // this field must `Some`
        let results = search_engine::SearchResults::<MusicInfoSearchItem> {
            hits: results
                .hits
                .into_iter()
                .map(|x| search_engine::SearchResult::<MusicInfoSearchItem> {
                    result: x.result,
                    matches_position: x
                        .matches_position
                        .unwrap()
                        .into_iter()
                        .map(|x| {
                            (
                                x.0,
                                x.1.into_iter()
                                    .map(|x| search_engine::MatchRange {
                                        start: x.start,
                                        length: x.length,
                                        indices: x.indices,
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                })
                .collect(),
            page: results.page.unwrap(),
            total_hits: results.total_hits.unwrap(),
            total_pages: results.total_pages.unwrap(),
        };
        Ok(results)
    }
}
