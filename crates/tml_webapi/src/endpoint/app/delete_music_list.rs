use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::app::delete_music_list;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub id: i64,
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
    match delete_music_list::handle(
        delete_music_list::Request {
            id: request_body.id,
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::app::delete_music_list::Repository::new(state.db),
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
                delete_music_list::Error::RepositoryError(error) => match error {
                    delete_music_list::repository::Error::MusicListNotFound => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The music list is not found".into(),
                            ))),
                        );
                    }
                    delete_music_list::repository::Error::PermissionDenied => {
                        return (
                            StatusCode::FORBIDDEN,
                            Json(ResponseBody::failed(Some("Permission denied".into()))),
                        );
                    }
                    delete_music_list::repository::Error::Unknown(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ResponseBody::failed(None)),
                        );
                    }
                },
            }
        }
    }
}
