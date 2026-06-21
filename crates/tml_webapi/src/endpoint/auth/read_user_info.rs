use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tml_application::usecase::auth::read_user_info;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Serialize)]
pub struct Data {
    pub id: i64,
    pub username: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub roles: Vec<String>,
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
) -> (StatusCode, Json<ResponseBody>) {
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
            Json(ResponseBody {
                success: true,
                message: None,
                data: Some(Data {
                    id: response.id,
                    username: response.username,
                    enabled: response.enabled,
                    created_at: response.created_at,
                    roles: response.roles,
                }),
            }),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                read_user_info::Error::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ResponseBody::failed(None)),
                ),
            }
        }
    }
}
