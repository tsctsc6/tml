use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::mgmt::read_job;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub id: i64,
}

#[derive(Serialize)]
pub struct Data {
    pub id: i64,
    pub job_type: String,
    pub job_args: serde_json::Value,
    pub status: String,
    pub description: String,
    pub error_message: String,
    pub success: bool,
    pub created_by_id: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseBody {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<Data>,
}

impl ResponseBody {
    fn failed(message: Option<String>) -> ResponseBody {
        ResponseBody {
            success: false,
            message,
            data: None,
        }
    }
}

fn format_iso8601(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.to_rfc3339()
}

fn format_opt_iso8601(dt: &Option<chrono::DateTime<chrono::Utc>>) -> Option<String> {
    dt.as_ref().map(|d| d.to_rfc3339())
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Query(query): axum::extract::Query<QueryParams>,
) -> (StatusCode, Json<ResponseBody>) {
    tracing::info!("Received request: {:?}", query);
    if !claims.inner.roles.iter().any(|role| role == "admin") {
        return (StatusCode::FORBIDDEN, Json(ResponseBody::failed(None)));
    }

    match read_job::handle(
        read_job::Request { id: query.id },
        &tml_infrastructure::usecase::mgmt::read_job::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(ResponseBody {
                success: true,
                message: None,
                data: Some(Data {
                    id: response.id,
                    job_type: format!("{:?}", response.job_type),
                    job_args: response.job_args,
                    status: format!("{:?}", response.status),
                    description: response.description,
                    error_message: response.error_message,
                    success: response.success,
                    created_by_id: response.created_by_id,
                    created_at: format_iso8601(&response.created_at),
                    completed_at: format_opt_iso8601(&response.completed_at),
                }),
            }),
        ),
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                read_job::Error::RepositoryError(e) => match e {
                    read_job::repository::Error::JobNotFound => (
                        StatusCode::OK,
                        Json(ResponseBody::failed(Some("Job not found".into()))),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ResponseBody::failed(None)),
                    ),
                },
            }
        }
    }
}
