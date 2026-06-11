use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tml_application::usecase::mgmt::delete_job;

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
    if !claims.inner.roles.iter().any(|role| role == "admin") {
        return (StatusCode::FORBIDDEN, Json(ResponseBody::failed(None)));
    }
    match delete_job::handle(
        delete_job::Request {
            id: request_body.id,
        },
        &tml_infrastructure::usecase::mgmt::delete_job::Repository::new(state.db),
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
                delete_job::Error::RepositoryError(error) => match error {
                    delete_job::repository::Error::JobNotFound => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some("The job is not found".into()))),
                        );
                    }
                    delete_job::repository::Error::Unknown(_) => {
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
