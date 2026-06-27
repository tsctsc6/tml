use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::mgmt::update_normal_user;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub id: i64,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(serde::Serialize)]
pub struct Data {}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<UnitizedResponseBody<Data>>) {
    tracing::info!("Received request: {:?}", request_body);
    if !claims.inner.roles.iter().any(|role| role == "admin") {
        return (
            StatusCode::FORBIDDEN,
            Json(UnitizedResponseBody::failed(None)),
        );
    }

    match update_normal_user::handle(
        update_normal_user::Request {
            id: request_body.id,
            username: request_body.username.as_deref(),
            password: request_body.password.as_deref(),
            enabled: request_body.enabled,
        },
        &state.password_hasher,
        &tml_infrastructure::usecase::mgmt::update_normal_user::Repository::new(
            state.db,
            state.user_id_security_stamp_cache,
        ),
    )
    .await
    {
        Ok(_) => (StatusCode::OK, Json(UnitizedResponseBody::success(Data {}))),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                update_normal_user::Error::ValidationError(error) => match error {
                    update_normal_user::validation::Error::UsernameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The username is too long".into(),
                            ))),
                        );
                    }
                    update_normal_user::validation::Error::UsernameTooShort => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The username is too short".into(),
                            ))),
                        );
                    }
                    update_normal_user::validation::Error::PasswordTooShort => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The password is too short".into(),
                            ))),
                        );
                    }
                    update_normal_user::validation::Error::NoFieldsToUpdate => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "No fields to update".into(),
                            ))),
                        );
                    }
                },
                update_normal_user::Error::HashingError(error) => match error {
                    _ => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UnitizedResponseBody::failed(None)),
                        );
                    }
                },
                update_normal_user::Error::RepositoryError(error) => match error {
                    update_normal_user::repository::Error::UserNotFound => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some("User not found".into()))),
                        );
                    }
                    update_normal_user::repository::Error::UserNotNormalUser => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "User is not a normal user".into(),
                            ))),
                        );
                    }
                    update_normal_user::repository::Error::UsernameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "Username already exists".into(),
                            ))),
                        );
                    }
                    update_normal_user::repository::Error::Unknown(_) => {
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
