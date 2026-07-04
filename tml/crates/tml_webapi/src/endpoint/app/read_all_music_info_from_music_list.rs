use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::app::read_all_music_info_from_music_list;

use crate::{app_state::AppState, endpoint::UnitizedResponseBody, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub music_list_id: i64,
    /// hex-encoded order cursor
    pub cursor: Option<String>,
    pub page_size: Option<u64>,
}

#[derive(Serialize)]
pub struct Item {
    /// hex-encoded music_info_id, 128 bit
    pub music_info_id: String,
    /// hex-encoded order (fractional index)
    pub order: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album_title: String,
    pub track_number: i32,
    pub audio_bitrate: i32,
    pub sample_rate: i32,
    pub channels: i16,
    pub bit_depth: i16,
}

#[derive(Serialize)]
pub struct Data {
    pub items: Vec<Item>,
    pub next_cursor: Option<String>,
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Query(query): axum::extract::Query<QueryParams>,
) -> (StatusCode, Json<UnitizedResponseBody<Data>>) {
    tracing::info!("Received request: {:?}", query);
    if !claims.inner.roles.iter().any(|role| role == "normal-user") {
        return (
            StatusCode::FORBIDDEN,
            Json(UnitizedResponseBody::failed(None)),
        );
    }

    // Decode hex cursor
    let cursor = match query.cursor {
        Some(ref hex_str) => match hex::decode(hex_str) {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                return (
                    StatusCode::OK,
                    Json(UnitizedResponseBody::failed(Some(
                        "Invalid cursor hex".into(),
                    ))),
                );
            }
        },
        None => None,
    };

    match read_all_music_info_from_music_list::handle(
        read_all_music_info_from_music_list::Request {
            music_list_id: query.music_list_id,
            cursor,
            page_size: query.page_size,
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::app::read_all_music_info_from_music_list::Repository::new(
            state.db,
        ),
    )
    .await
    {
        Ok(response) => {
            let items: Vec<Item> = response
                .items
                .into_iter()
                .map(|x| Item {
                    music_info_id: hex::encode(&x.music_info_id),
                    order: hex::encode(&x.order),
                    title: x.title,
                    artists: x.artists,
                    album_title: x.album_title,
                    track_number: x.track_number,
                    audio_bitrate: x.audio_bitrate,
                    sample_rate: x.sample_rate,
                    channels: x.channels,
                    bit_depth: x.bit_depth,
                })
                .collect();
            let next_cursor = response.next_cursor.map(|c| hex::encode(&c));
            (
                StatusCode::OK,
                Json(UnitizedResponseBody::success(Data { items, next_cursor })),
            )
        }
        Err(e) => {
            tracing::warn!("Error occurred: {}", e);
            match e {
                read_all_music_info_from_music_list::Error::RepositoryError(error) => match error {
                    read_all_music_info_from_music_list::repository::Error::MusicListNotFound => {
                        return (
                            StatusCode::OK,
                            Json(UnitizedResponseBody::failed(Some(
                                "The music list is not found".into(),
                            ))),
                        );
                    }
                    read_all_music_info_from_music_list::repository::Error::Unknown(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UnitizedResponseBody::failed(None)),
                        );
                    }
                },
                read_all_music_info_from_music_list::Error::PermissionDenied => {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(UnitizedResponseBody::failed(Some(
                            "Permission denied".into(),
                        ))),
                    );
                }
                read_all_music_info_from_music_list::Error::PageSizeOutOfRange => {
                    return (
                        StatusCode::OK,
                        Json(UnitizedResponseBody::failed(Some(
                            "Page size out of range".into(),
                        ))),
                    );
                }
            }
        }
    }
}
