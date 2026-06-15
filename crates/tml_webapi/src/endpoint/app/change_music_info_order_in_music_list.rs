use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::app::change_music_info_order_in_music_list;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub music_list_id: i64,
    /// hex-encoded music_info_id, 128 bit
    pub music_info_id: String,
    /// hex-encoded music_info_id, 128 bit, the previous item of target location 
    pub prev_music_info_id: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ResponseBody {
    pub success: bool,
    pub message: Option<String>,
}

impl ResponseBody {
    fn failed(message: Option<String>) -> ResponseBody {
        ResponseBody {
            success: false,
            message,
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
    if !claims.inner.roles.iter().any(|role| role == "normal-user") {
        return (StatusCode::FORBIDDEN, Json(ResponseBody::failed(None)));
    }

    // Decode hex music_info_id
    let music_info_id = match hex::decode(&request_body.music_info_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(ResponseBody::failed(Some(
                    "Invalid music_info_id hex".into(),
                ))),
            );
        }
    };

    // Decode hex prev_music_info_id
    let prev_music_info_id = match request_body.prev_music_info_id {
        Some(ref hex_str) => match hex::decode(hex_str) {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                return (
                    StatusCode::OK,
                    Json(ResponseBody::failed(Some(
                        "Invalid prev_music_info_id hex".into(),
                    ))),
                );
            }
        },
        None => None,
    };

    match change_music_info_order_in_music_list::handle(
        change_music_info_order_in_music_list::Request {
            music_list_id: request_body.music_list_id,
            music_info_id: &music_info_id,
            prev_music_info_id: prev_music_info_id.as_deref(),
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::app::change_music_info_order_in_music_list::Repository::new(
            state.db,
        ),
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: true,
                message: None,
            }),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                change_music_info_order_in_music_list::Error::RepositoryError(error) => {
                    match error {
                        change_music_info_order_in_music_list::repository::Error::MusicListNotFound => {
                            return (
                                StatusCode::OK,
                                Json(ResponseBody::failed(Some(
                                    "The music list is not found".into(),
                                ))),
                            );
                        }
                        change_music_info_order_in_music_list::repository::Error::MusicInfoNotInMusicList => {
                            return (
                                StatusCode::OK,
                                Json(ResponseBody::failed(Some(
                                    "The music info is not in the music list".into(),
                                ))),
                            );
                        }
                        change_music_info_order_in_music_list::repository::Error::Unknown(_) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ResponseBody::failed(None)),
                            );
                        }
                    }
                }
                change_music_info_order_in_music_list::Error::PermissionDenied => {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ResponseBody::failed(Some("Permission denied".into()))),
                    );
                }
                change_music_info_order_in_music_list::Error::DecodeError(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ResponseBody::failed(None)),
                    );
                }
                change_music_info_order_in_music_list::Error::InvalidReorder => {
                    return (
                        StatusCode::OK,
                        Json(ResponseBody::failed(Some(
                            "Invalid reorder: cannot place item at the specified position".into(),
                        ))),
                    );
                }
            }
        }
    }
}
