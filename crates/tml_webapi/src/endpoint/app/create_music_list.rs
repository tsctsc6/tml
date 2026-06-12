use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::app::create_music_list;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub name: String,
}

#[derive(Serialize)]
pub struct ResponseBody {
    pub success: bool,
    pub message: Option<String>,
    pub id: Option<i64>,
}

impl ResponseBody {
    fn failed(message: Option<String>) -> ResponseBody {
        ResponseBody {
            success: false,
            message,
            id: None,
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
    match create_music_list::handle(
        create_music_list::Request {
            name: &request_body.name,
            user_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::app::create_music_list::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: true,
                id: Some(response.id),
                message: None,
            }),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                create_music_list::Error::ValidationError(error) => match error {
                    create_music_list::validation::Error::NameEmpty => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The name is empty".into()))),
                        );
                    }
                    create_music_list::validation::Error::NameTooLong => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The name is too long".into()))),
                        );
                    }
                },
                create_music_list::Error::RepositoryError(error) => match error {
                    create_music_list::repository::Error::NameDuplication => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The name is already exists".into(),
                            ))),
                        );
                    }
                    create_music_list::repository::Error::Unknown(_) => {
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
