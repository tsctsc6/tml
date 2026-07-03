use std::path::Path;

use axum::{
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use axum_extra::headers::Range;
use axum_range::{KnownSize, Ranged};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use tokio::fs::File;

use tml_application::usecase::app::get_music_info_file_path;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    /// hex-encoded music_info_id
    pub id: String,
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    range: Option<TypedHeader<Range>>,
    axum::extract::Query(query): axum::extract::Query<QueryParams>,
) -> impl IntoResponse {
    tracing::debug!("Range: {:?}", range);
    tracing::info!("Received request: {:?}", query);
    if !claims.inner.roles.iter().any(|role| role == "normal-user") {
        return StatusCode::FORBIDDEN.into_response();
    }

    let music_info_id = match hex::decode(&query.id) {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("{}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let file_path_str = match get_music_info_file_path::handle(
        get_music_info_file_path::Request { music_info_id },
        &tml_infrastructure::usecase::app::get_music_info_file_path::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => response.file_path,
        Err(e) => {
            tracing::warn!("{}", e);
            return match e {
                get_music_info_file_path::Error::RepositoryError(err) => match err {
                    get_music_info_file_path::repository::Error::MusicInfoNotFound => {
                        StatusCode::NOT_FOUND.into_response()
                    }
                    get_music_info_file_path::repository::Error::StorageNotFound => {
                        StatusCode::NOT_FOUND.into_response()
                    }
                    get_music_info_file_path::repository::Error::Unknown(_) => {
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                },
            };
        }
    };

    let file_path = Path::new(&file_path_str);
    let file = match File::open(file_path).await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("{}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };
    let body = match KnownSize::file(file).await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let ranged = Ranged::new(range.map(|r| r.0), body);

    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    let file_name: String = utf8_percent_encode(file_name, NON_ALPHANUMERIC).collect();
    let content_disposition = format!("attachment; filename*=UTF-8''{}", file_name);

    (
        [
            (header::CONTENT_TYPE, "video/x-flac"),
            (header::CONTENT_DISPOSITION, content_disposition.as_str()),
        ],
        ranged,
    )
        .into_response()
}
