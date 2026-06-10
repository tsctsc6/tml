use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::app_trait;
use tml_application::usecase::auth::login;

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
    pub token: Option<String>,
}

impl ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody {
            success: false,
            message: None,
            token: None,
        }
    }
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<ResponseBody>) {
    tracing::info!("Received request: {}", request_body.username);
    match login::handle(
        login::Request {
            username: &request_body.username,
            password: &request_body.password,
        },
        &state.password_hasher,
        &state.jwt_manager,
        &tml_infrastructure::usecase::auth::login::Repository::new(
            state.db,
            state.user_id_security_stamp_cache,
        ),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: true,
                message: None,
                token: response.token,
            }),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                login::Error::RepositoryError(error) => {
                    if let login::repository::Error::UserNotFound = error {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody {
                                success: false,
                                message: Some("User not found".into()),
                                token: None,
                            }),
                        );
                    }
                }
                login::Error::PasswordHasherError(error) => {
                    if let app_trait::password_hasher::Error::InvalidPassword = error {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody {
                                success: false,
                                message: Some("Invalid password".into()),
                                token: None,
                            }),
                        );
                    }
                }
                login::Error::UserDisabled => {
                    return (
                        StatusCode::OK,
                        Json(ResponseBody {
                            success: false,
                            message: Some("User disabled".into()),
                            token: None,
                        }),
                    );
                }
                _ => (),
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseBody::default()),
            )
        }
    }
}
