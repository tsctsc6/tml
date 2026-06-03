use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::auth::register;

use crate::AppState;

#[derive(Deserialize)]
pub struct RequestBody {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct ResponseBody {
    pub success: bool,
    pub message: Option<String>,
    pub id: Option<i64>,
}

impl ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody {
            success: false,
            message: None,
            id: None,
        }
    }
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<ResponseBody>) {
    tracing::info!("Received request: {}", request_body.username);
    match register::handle(
        register::Request {
            username: &request_body.username,
            password: &request_body.password,
        },
        &*state.password_hasher,
        &tml_infrastructure::usecase::auth::register::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: true,
                id: Some(response.id),
                message: None,
            }),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            if let register::Error::RepositoryError(error) = e {
                if let register::repository::Error::UniqueIndex(_) = error {
                    return (
                        StatusCode::OK,
                        Json(ResponseBody {
                            success: false,
                            message: Some("User already exists".into()),
                            id: None,
                        }),
                    );
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseBody::default()),
            )
        }
    }
}
