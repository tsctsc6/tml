use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tml_application::usecase::auth::read_user_info;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Serialize)]
pub struct Data {
    pub id: i64,
    pub username: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub roles: Vec<String>,
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
) -> (StatusCode, Json<UnitizedResponseBody<Data>>) {
    tracing::info!(
        "Received request: read_user_info for user_id={}",
        claims.inner.sub
    );
    match read_user_info::handle(
        read_user_info::Request {
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::auth::read_user_info::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(UnitizedResponseBody::success(Data {
                id: response.id,
                username: response.username,
                enabled: response.enabled,
                created_at: response.created_at,
                roles: response.roles,
            })),
        ),
        Err(e) => {
            tracing::warn!("Error occurred: {}", e);
            match e {
                read_user_info::Error::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UnitizedResponseBody::failed(None)),
                ),
            }
        }
    }
}
