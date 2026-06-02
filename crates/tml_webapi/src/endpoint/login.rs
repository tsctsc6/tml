use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::app_trait;
use tml_application::usecase::auth::login;

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
    token: Option<String>,
}

impl ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody {
            success: false,
            message: None,
            token: None,
        }
    }
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    Json(request_body): Json<RequestBody>,
) -> (StatusCode, Json<ResponseBody>) {
    match login::handle(
        login::Request {
            username: request_body.username,
            password: request_body.password,
        },
        &*state.password_hasher,
        &*state.jwt_manager,
        &tml_infrastructure::usecase::auth::login::Repository::new(
            state.db,
            state.user_id_security_stamp_cache,
        ),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: true,
                message: None,
                token: response.token,
            }),
        ),
        Err(e) => {
            eprintln!("Error occurred: {}", e);
            match e {
                login::Error::RepositoryError(error) => match error {
                    login::repository::Error::UserNotFound => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody {
                                success: false,
                                message: Some("User not found".into()),
                                token: None,
                            }),
                        );
                    }
                    _ => (),
                },
                login::Error::PasswordHasherError(error) => match error {
                    app_trait::password_hasher::Error::InvalidPassword => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody {
                                success: false,
                                message: Some("Invalid password".into()),
                                token: None,
                            }),
                        );
                    }
                    _ => (),
                },
                _ => (),
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseBody::default()),
            )
        }
    }
}
