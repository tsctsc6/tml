use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::mgmt::create_normal_user;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
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
    fn failed(message: Option<String>) -> ResponseBody {
        ResponseBody {
            success: false,
            message,
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
        return (StatusCode::FORBIDDEN, Json(ResponseBody::failed(None)));
    }

    match create_normal_user::handle(
        create_normal_user::Request {
            username: &request_body.username,
            password: &request_body.password,
        },
        &state.password_hasher,
        &tml_infrastructure::usecase::mgmt::create_normal_user::Repository::new(state.db),
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
                create_normal_user::Error::ValidationError(error) => match error {
                    create_normal_user::validation::Error::UsernameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The username is too long".into()))),
                        );
                    }
                    create_normal_user::validation::Error::UsernameTooShort => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The username is too short".into()))),
                        );
                    }
                    create_normal_user::validation::Error::PasswordTooShort => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The password is too short".into(),
                            ))),
                        );
                    }
                },
                create_normal_user::Error::HashingError(error) => match error {
                    _ => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ResponseBody::failed(None)),
                        );
                    }
                },
                create_normal_user::Error::RepositoryError(error) => match error {
                    create_normal_user::repository::Error::UsernameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("User already exists".into()))),
                        );
                    }
                    create_normal_user::repository::Error::Unknown(_) => {
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
