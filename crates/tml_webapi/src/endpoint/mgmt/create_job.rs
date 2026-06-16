use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::mgmt::create_job;
use tml_domain::model::mgmt::job;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub job_type: String,
    pub job_args: Option<serde_json::Value>,
    pub description: Option<String>,
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
    if !claims.inner.roles.iter().any(|role| role == "admin") {
        return (StatusCode::FORBIDDEN, Json(ResponseBody::failed(None)));
    }

    let job_type = match request_body.job_type.as_str() {
        "scan_incremental" => job::JobType::ScanIncremental,
        "build_index" => job::JobType::BuildIndex,
        "update_index" => job::JobType::UpdateIndex,
        _ => {
            return (
                StatusCode::OK,
                Json(ResponseBody::failed(Some("Invalid job_type".into()))),
            );
        }
    };

    let job_args = request_body.job_args.unwrap_or(serde_json::Value::Null);
    let description = request_body.description.unwrap_or_default();

    match create_job::handle(
        create_job::Request {
            job_type: &job_type,
            job_args: &job_args,
            description: &description,
            created_by_id: claims.inner.sub,
        },
        &tml_infrastructure::usecase::mgmt::create_job::Repository::new(state.db.clone()),
        &tml_infrastructure::job_handler::JobHandler::new(
            tml_infrastructure::job_handler::Repository::new(state.db),
            state.music_info_provider,
            state.meilisearch_client,
        ),
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
                create_job::Error::ValidationError(error) => match error {
                    create_job::validation::Error::DescriptionTooLong => {
                        return (
                            StatusCode::OK,
                            Json(ResponseBody::failed(Some(
                                "The description is too long".into(),
                            ))),
                        );
                    }
                },
                create_job::Error::RepositoryError(error) => match error {
                    create_job::repository::Error::Unknown(_) => {
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
