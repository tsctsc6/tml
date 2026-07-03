use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::mgmt::create_storage;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub name: String,
    pub path: String,
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
        return (
            StatusCode::FORBIDDEN,
            Json(UnitizedResponseBody::failed(None)),
        );
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
            Json(UnitizedResponseBody::success(Data { id: response.id })),
        ),
        Err(e) => {
            tracing::warn!("Error occurred: {}", e);
            match e {
                create_storage::Error::ValidationError(error) => match error {
                    create_storage::validation::Error::NameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The name is too long".into(),
                            ))),
                        );
                    }
                    create_storage::validation::Error::DirectoryNotExist => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The directory does not exist or is a file.".into(),
                            ))),
                        );
                    }
                    create_storage::validation::Error::PathIsRelative => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The path is relative".into(),
                            ))),
                        );
                    }
                },
                create_storage::Error::RepositoryError(error) => match error {
                    create_storage::repository::Error::NameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The name is already exists".into(),
                            ))),
                        );
                    }
                    create_storage::repository::Error::PathDuplication => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The path is already exists".into(),
                            ))),
                        );
                    }
                    create_storage::repository::Error::Unknown(_) => {
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
