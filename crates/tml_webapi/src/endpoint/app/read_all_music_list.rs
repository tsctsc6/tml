use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::app::read_all_music_list;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub cursor: Option<i64>,
    pub page_size: Option<u64>,
}

#[derive(Serialize)]
pub struct Item {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct Data {
    pub items: Vec<Item>,
    pub next_cursor: Option<i64>,
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Query(query): axum::extract::Query<QueryParams>,
) -> (StatusCode, Json<UnitizedResponseBody<Data>>) {
    tracing::info!("Received request: {:?}", query);
    if !claims.inner.roles.iter().any(|role| role == "normal-user") {
        return (StatusCode::FORBIDDEN, Json(UnitizedResponseBody::failed(None)));
    }

    match read_all_music_list::handle(
        read_all_music_list::Request {
            cursor: query.cursor,
            page_size: query.page_size,
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::app::read_all_music_list::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(UnitizedResponseBody::success(Data {
                items: response
                    .items
                    .into_iter()
                    .map(|item| Item {
                        id: item.id,
                        name: item.name,
                    })
                    .collect(),
                next_cursor: response.next_cursor,
            })),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                read_all_music_list::Error::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UnitizedResponseBody::failed(None)),
                ),
                read_all_music_list::Error::PageSizeOutOfRange => (
                    StatusCode::OK,
                    Json(UnitizedResponseBody::failed(Some("Page size out of range".into()))),
                ),
            }
        }
    }
}
