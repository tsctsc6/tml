use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::auth::register;

use crate::AppState;

#[derive(Deserialize)]
pub struct RequestBody {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct ResponseBody {
    success: bool,
    message: Option<String>,
    id: Option<i64>,
}

impl ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody {
            success: false,
            message: None,
            id: None,
        }
    }
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<ResponseBody>) {
    match register::handle(
        register::Request {
            username: request_body.username,
            password: request_body.password,
        },
        &*state.password_hasher,
        &tml_infrastructure::usecase::auth::register::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: response.success,
                id: Some(response.id),
                message: response.message,
            }),
        ),
        Err(e) => {
            eprintln!("Error occurred: {}", e);
            match e {
                register::Error::RepositoryError(error) => match error {
                    register::repository::Error::UniqueIndex(_) => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody {
                                success: false,
                                message: Some("User already exists".into()),
                                id: None,
                            }),
                        );
                    }
                    _ => {}
                },
                _ => {}
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseBody::default()),
            )
        }
    }
}
