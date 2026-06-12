use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::app::update_music_list;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub id: i64,
    pub name: String,
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
    match update_music_list::handle(
        update_music_list::Request {
            id: request_body.id,
            name: &request_body.name,
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::app::update_music_list::Repository::new(state.db),
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
                update_music_list::Error::ValidationError(error) => match error {
                    update_music_list::validation::Error::NameEmpty => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The name is empty".into()))),
                        );
                    }
                    update_music_list::validation::Error::NameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The name is too long".into()))),
                        );
                    }
                },
                update_music_list::Error::RepositoryError(error) => match error {
                    update_music_list::repository::Error::NameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The name is already exists".into(),
                            ))),
                        );
                    }
                    update_music_list::repository::Error::MusicListNotFound => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The music list is not found".into(),
                            ))),
                        );
                    }
                    update_music_list::repository::Error::Unknown(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ResponseBody::failed(None)),
                        );
                    }
                },
                update_music_list::Error::PermissionDenied => {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ResponseBody::failed(Some("Permission denied".into()))),
                    );
                }
            }
        }
    }
}
