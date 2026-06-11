use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tml_application::usecase::mgmt::read_all_job;

use crate::{app_state::AppState, extractor::Claims};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub cursor: Option<String>,
    pub page_size: Option<u64>,
    /// ISO 8601 format, target_time >= created_after
    pub created_after: Option<String>,
    /// ISO 8601 format, target_time <= created_before
    pub created_before: Option<String>,
}

#[derive(Serialize)]
pub struct Item {
    pub id: i64,
    pub job_type: String,
    pub status: String,
    pub success: bool,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Serialize)]
pub struct Data {
    pub items: Vec<Item>,
    pub next_cursor: Option<String>,
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

fn parse_iso8601(s: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                .map(|naive| naive.and_utc())
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|naive| naive.and_utc())
        })
        .map_err(|e| format!("Invalid datetime format: {}", e))
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

    let page_size = query.page_size.unwrap_or(10);

    let cursor = match query.cursor.as_deref() {
        Some(s) => match parse_iso8601(s) {
            Ok(dt) => Some(dt),
            Err(e) => {
                return (StatusCode::OK, Json(ResponseBody::failed(Some(e))));
            }
        },
        None => None,
    };

    let created_after = match query.created_after.as_deref() {
        Some(s) => match parse_iso8601(s) {
            Ok(dt) => Some(dt),
            Err(e) => {
                return (StatusCode::OK, Json(ResponseBody::failed(Some(e))));
            }
        },
        None => None,
    };

    let created_before = match query.created_before.as_deref() {
        Some(s) => match parse_iso8601(s) {
            Ok(dt) => Some(dt),
            Err(e) => {
                return (StatusCode::OK, Json(ResponseBody::failed(Some(e))));
            }
        },
        None => None,
    };

    match read_all_job::handle(
        read_all_job::Request {
            cursor,
            page_size,
            created_after,
            created_before,
        },
        &tml_infrastructure::usecase::mgmt::read_all_job::Repository::new(state.db),
    )
    .await
    {
        Ok(response) => {
            let next_cursor = response.next_cursor.map(|dt| format_iso8601(&dt));
            (
                StatusCode::OK,
                Json(ResponseBody {
                    success: true,
                    message: None,
                    data: Some(Data {
                        items: response
                            .items
                            .into_iter()
                            .map(|item| Item {
                                id: item.id,
                                job_type: format!("{:?}", item.job_type),
                                status: format!("{:?}", item.status),
                                success: item.success,
                                created_at: format_iso8601(&item.created_at),
                                completed_at: format_opt_iso8601(&item.completed_at),
                            })
                            .collect(),
                        next_cursor,
                    }),
                }),
            )
        }
        Err(e) => {
            tracing::error!("Error occurred: {}", e);
            match e {
                read_all_job::Error::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ResponseBody::failed(None)),
                ),
                read_all_job::Error::PageSizeOutOfRange => (
                    StatusCode::OK,
                    Json(ResponseBody::failed(Some("Page size out of range".into()))),
                ),
                read_all_job::Error::DateTimeOutOfRange => (
                    StatusCode::OK,
                    Json(ResponseBody::failed(Some("Datetime out of range".into()))),
                ),
            }
        }
    }
}
