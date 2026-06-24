use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::app_trait;
use tml_application::usecase::auth::login;

use crate::AppState;
use crate::endpoint::UnitizedResponseBody;

#[derive(Deserialize)]
pub struct RequestBody {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct Data {
    pub token: String,
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<UnitizedResponseBody<Data>>) {
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
            Json(UnitizedResponseBody::success(Data {
                token: response.token,
            })),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                login::Error::RepositoryError(error) => {
                    if let login::repository::Error::UserNotFound = error {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some("User not found".into()))),
                        );
                    }
                }
                login::Error::PasswordHasherError(error) => {
                    if let app_trait::password_hasher::Error::InvalidPassword = error {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "Invalid password".into(),
                            ))),
                        );
                    }
                }
                login::Error::UserDisabled => {
                    return (
                        StatusCode::OK,
                        Json(UnitizedResponseBody::failed(Some("User disabled".into()))),
                    );
                }
                _ => (),
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UnitizedResponseBody::failed(None)),
            )
        }
    }
}
