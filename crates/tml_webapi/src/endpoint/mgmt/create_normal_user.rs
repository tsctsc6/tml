use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::mgmt::create_normal_user;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct Data {
    pub id: i64,
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<UnitizedResponseBody<Data>>) {
    tracing::info!("Received request: {:?}", request_body);
    if !claims.inner.roles.iter().any(|role| role == "admin") {
        return (StatusCode::FORBIDDEN, Json(UnitizedResponseBody::failed(None)));
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
            Json(UnitizedResponseBody::success(Data {
                id: response.id,
            })),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                create_normal_user::Error::ValidationError(error) => match error {
                    create_normal_user::validation::Error::UsernameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The username is too long".into(),
                            ))),
                        );
                    }
                    create_normal_user::validation::Error::UsernameTooShort => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The username is too short".into(),
                            ))),
                        );
                    }
                    create_normal_user::validation::Error::PasswordTooShort => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The password is too short".into(),
                            ))),
                        );
                    }
                },
                create_normal_user::Error::HashingError(error) => match error {
                    _ => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UnitizedResponseBody::failed(None)),
                        );
                    }
                },
                create_normal_user::Error::RepositoryError(error) => match error {
                    create_normal_user::repository::Error::UsernameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some("User already exists".into()))),
                        );
                    }
                    create_normal_user::repository::Error::Unknown(_) => {
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
