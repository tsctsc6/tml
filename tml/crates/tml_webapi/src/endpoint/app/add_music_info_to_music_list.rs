use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::app::add_music_info_to_music_list;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub music_list_id: i64,
    /// hex-encoded music_info_id, 128 bit
    pub music_info_id: String,
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

    // Decode hex music_info_id
    let music_info_id = match hex::decode(&request_body.music_info_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(UnitizedResponseBody::failed(Some(
                    "Invalid music_info_id hex".into(),
                ))),
            );
        }
    };

    match add_music_info_to_music_list::handle(
        add_music_info_to_music_list::Request {
            music_list_id: request_body.music_list_id,
            music_info_id: &music_info_id,
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::app::add_music_info_to_music_list::Repository::new(),
        &tml_infrastructure::tx_context::SeaOrmTxManager::new(state.db),
    )
    .await
    {
        Ok(_) => (StatusCode::OK, Json(UnitizedResponseBody::success(Data {}))),
        Err(e) => {
            tracing::warn!("Error occurred: {}", e);
            match e {
                add_music_info_to_music_list::Error::RepositoryError(error) => match error {
                    add_music_info_to_music_list::repository::Error::MusicListNotFound => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The music list is not found".into(),
                            ))),
                        );
                    }
                    add_music_info_to_music_list::repository::Error::MusicInfoNotFound => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The music info is not found".into(),
                            ))),
                        );
                    }
                    add_music_info_to_music_list::repository::Error::MusicInfoAlreadyInMusicList => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The music info is already in the music list".into(),
                            ))),
                        );
                    }
                    add_music_info_to_music_list::repository::Error::Unknown(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UnitizedResponseBody::failed(None)),
                        );
                    }
                },
                add_music_info_to_music_list::Error::PermissionDenied => {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(UnitizedResponseBody::failed(Some("Permission denied".into()))),
                    );
                },
                add_music_info_to_music_list::Error::DecodeError(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UnitizedResponseBody::failed(None)),
                    )
                },
                add_music_info_to_music_list::Error::TxError(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UnitizedResponseBody::failed(None)),
                    )
                },
            }
        }
    }
}
