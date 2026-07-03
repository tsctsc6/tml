use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::auth::update_user;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(serde::Serialize)]
pub struct Data {}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<UnitizedResponseBody<Data>>) {
    tracing::info!(
        "Received request: update_user for user_id={}",
        claims.inner.sub
    );
    match update_user::handle(
        update_user::Request {
            user_id: claims.inner.sub,
            username: request_body.username.as_deref(),
            password: request_body.password.as_deref(),
        },
        &state.password_hasher,
        &tml_infrastructure::usecase::auth::update_user::Repository::new(
            state.db,
            state.user_id_security_stamp_cache,
        ),
    )
    .await
    {
        Ok(_) => (StatusCode::OK, Json(UnitizedResponseBody::success(Data {}))),
        Err(e) => {
            tracing::warn!("Error occurred: {}", e);
            match e {
                update_user::Error::ValidationError(error) => match error {
                    update_user::validation::Error::UsernameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The username is too long".into(),
                            ))),
                        );
                    }
                    update_user::validation::Error::UsernameTooShort => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The username is too short".into(),
                            ))),
                        );
                    }
                    update_user::validation::Error::PasswordTooShort => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The password is too short".into(),
                            ))),
                        );
                    }
                    update_user::validation::Error::NoFieldsToUpdate => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "No fields to update".into(),
                            ))),
                        );
                    }
                },
                update_user::Error::HashingError(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UnitizedResponseBody::failed(None)),
                    );
                }
                update_user::Error::RepositoryError(error) => match error {
                    update_user::repository::Error::UserNotFound => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some("User not found".into()))),
                        );
                    }
                    update_user::repository::Error::UsernameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "Username already exists".into(),
                            ))),
                        );
                    }
                    update_user::repository::Error::Unknown(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UnitizedResponseBody::failed(None)),
                        );
                    }
                },
            }
        }
    }
}
