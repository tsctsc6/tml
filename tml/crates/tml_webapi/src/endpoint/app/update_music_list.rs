use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::app::update_music_list;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub id: i64,
    pub name: String,
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
    if !claims.inner.roles.iter().any(|role| role == "normal-user") {
        return (
            StatusCode::FORBIDDEN,
            Json(UnitizedResponseBody::failed(None)),
        );
    }
    match update_music_list::handle(
        update_music_list::Request {
            id: request_body.id,
            name: &request_body.name,
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::app::update_music_list::Repository::new(),
        &tml_infrastructure::tx_context::SeaOrmTxManager::new(state.db),
    )
    .await
    {
        Ok(_) => (StatusCode::OK, Json(UnitizedResponseBody::success(Data {}))),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                update_music_list::Error::ValidationError(error) => match error {
                    update_music_list::validation::Error::NameEmpty => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The name is empty".into(),
                            ))),
                        );
                    }
                    update_music_list::validation::Error::NameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The name is too long".into(),
                            ))),
                        );
                    }
                },
                update_music_list::Error::RepositoryError(error) => match error {
                    update_music_list::repository::Error::NameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The name is already exists".into(),
                            ))),
                        );
                    }
                    update_music_list::repository::Error::MusicListNotFound => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The music list is not found".into(),
                            ))),
                        );
                    }
                    update_music_list::repository::Error::Unknown(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UnitizedResponseBody::failed(None)),
                        );
                    }
                },
                update_music_list::Error::PermissionDenied => {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(UnitizedResponseBody::failed(Some(
                            "Permission denied".into(),
                        ))),
                    );
                }
                update_music_list::Error::TxError(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UnitizedResponseBody::failed(None)),
                    );
                }
            }
        }
    }
}
