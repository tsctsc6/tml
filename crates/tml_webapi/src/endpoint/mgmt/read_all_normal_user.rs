use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::mgmt::read_all_normal_user;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub page_index: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Serialize)]
pub struct Item {
    pub id: i64,
    pub username: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct Data {
    pub total: u64,
    pub items: Vec<Item>,
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Query(query): axum::extract::Query<QueryParams>,
) -> (StatusCode, Json<UnitizedResponseBody<Data>>) {
    tracing::info!("Received request: {:?}", query);
    if !claims.inner.roles.iter().any(|role| role == "admin") {
        return (StatusCode::FORBIDDEN, Json(UnitizedResponseBody::failed(None)));
    }
    let page_index = query.page_index.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(10);
    match read_all_normal_user::handle(
        read_all_normal_user::Request {
            page_index,
            page_size,
        },
        &tml_infrastructure::usecase::mgmt::read_all_normal_user::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(UnitizedResponseBody::success(Data {
                total: response.total,
                items: response
                    .items
                    .into_iter()
                    .map(|item| Item {
                        id: item.id,
                        username: item.username,
                        enabled: item.enabled,
                        created_at: item.created_at,
                    })
                    .collect(),
            })),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                read_all_normal_user::Error::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UnitizedResponseBody::failed(None)),
                ),
                read_all_normal_user::Error::PageSizeOutOfRange => (
                    StatusCode::OK,
                    Json(UnitizedResponseBody::failed(Some("Page size out of range".into()))),
                ),
            }
        }
    }
}
