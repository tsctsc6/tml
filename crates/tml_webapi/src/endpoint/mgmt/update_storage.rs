use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::mgmt::update_storage;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub id: i64,
    pub name: String,
    pub path: String,
}

#[derive(serde::Serialize)]
pub struct ResponseBody {
    pub success: bool,
    pub message: Option<String>,
}

impl ResponseBody {
    fn failed(message: Option<String>) -> ResponseBody {
        ResponseBody {
            success: false,
            message,
        }
    }
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<ResponseBody>) {
    tracing::info!("Received request: {:?}", request_body);
    if !claims.inner.roles.iter().any(|role| role == "admin") {
        return (StatusCode::FORBIDDEN, Json(ResponseBody::failed(None)));
    }
    match update_storage::handle(
        update_storage::Request {
            id: request_body.id,
            name: &request_body.name,
            path: &request_body.path,
        },
        &tml_infrastructure::usecase::mgmt::update_storage::Repository::new(state.db),
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: true,
                message: None,
            }),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                update_storage::Error::ValidationError(error) => match error {
                    update_storage::validation::Error::NameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The name is too long".into()))),
                        );
                    }
                    update_storage::validation::Error::InvalidPath => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The path is invalid".into()))),
                        );
                    }
                },
                update_storage::Error::RepositoryError(error) => match error {
                    update_storage::repository::Error::NameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The name is already exists".into(),
                            ))),
                        );
                    }
                    update_storage::repository::Error::PathDuplication => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The path is already exists".into(),
                            ))),
                        );
                    }
                    update_storage::repository::Error::StorageNotFound => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The storage is not found".into(),
                            ))),
                        );
                    }
                    update_storage::repository::Error::Unknown(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ResponseBody::failed(None)),
                        );
                    }
                },
            }
        }
    }
}
