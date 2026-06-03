use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::mgmt::create_storage;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub name: String,
    pub path: String,
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
    claims: Claims,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<ResponseBody>) {
    tracing::info!("Received request: {:?}", request_body);
    if !claims.inner.roles.iter().any(|role| role == "admin") {
        return (StatusCode::FORBIDDEN, Json(ResponseBody::default()));
    }
    match create_storage::handle(
        create_storage::Request {
            name: &request_body.name,
            path: &request_body.path,
        },
        &tml_infrastructure::usecase::mgmt::create_storage::Repository::new(state.db),
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
            match e {
                create_storage::Error::ValidationError(error) => match error {
                    create_storage::validation::Error::NameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody {
                                success: false,
                                message: Some("The name is too long".into()),
                                id: None,
                            }),
                        );
                    }
                    create_storage::validation::Error::InvalidPath => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody {
                                success: false,
                                message: Some("The path is invalid".into()),
                                id: None,
                            }),
                        );
                    }
                },
                create_storage::Error::RepositoryError(error) => match error {
                    create_storage::repository::Error::NameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody {
                                success: false,
                                message: Some("The name is already exists".into()),
                                id: None,
                            }),
                        );
                    }
                    create_storage::repository::Error::Unknown(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ResponseBody::default()),
                        );
                    }
                },
            }
        }
    }
}
