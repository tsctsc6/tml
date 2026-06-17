use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::{
    app_trait::search_engine::{SearchResult, SearchResults},
    usecase::app::search_music_info,
};

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub query: String,
    pub hits_per_page: Option<usize>,
    pub page: Option<usize>,
}

#[derive(Serialize)]
pub struct Item {
    /// hex-encoded, 128 bit
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album_title: String,
}

#[derive(Serialize)]
pub struct Data {
    #[serde(flatten)]
    pub items: SearchResults<Item>,
}

#[derive(Serialize)]
pub struct ResponseBody {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<Data>,
}

impl ResponseBody {
    fn failed(message: Option<String>) -> ResponseBody {
        ResponseBody {
            success: false,
            message,
            data: None,
        }
    }
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Query(query): axum::extract::Query<QueryParams>,
) -> (StatusCode, Json<ResponseBody>) {
    tracing::info!("Received request: {:?}", query);
    if !claims.inner.roles.iter().any(|role| role == "normal-user") {
        return (StatusCode::FORBIDDEN, Json(ResponseBody::failed(None)));
    }

    match search_music_info::handle(
        search_music_info::Request {
            query: &query.query,
            hits_per_page: query.hits_per_page,
            page: query.page,
        },
        &tml_infrastructure::search_engine::SearchEngine::new(
            state.meilisearch_client,
            &state.app_config.meilisearch.index_name,
        ),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: true,
                message: None,
                data: Some(Data {
                    items: SearchResults::<Item> {
                        hits: response
                            .results
                            .hits
                            .into_iter()
                            .map(|x| SearchResult::<Item> {
                                result: Item {
                                    id: x.result.id,
                                    title: x.result.title,
                                    artists: x.result.artists,
                                    album_title: x.result.album_title,
                                },
                                matches_position: x.matches_position,
                            })
                            .collect(),
                        page: response.results.page,
                        total_hits: response.results.total_hits,
                        total_pages: response.results.total_pages,
                    },
                }),
            }),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseBody::failed(None)),
            )
        }
    }
}
