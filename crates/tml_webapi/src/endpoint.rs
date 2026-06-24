use serde::Serialize;

pub mod app;
pub mod auth;
pub mod mgmt;

#[derive(Serialize)]
pub struct UnitizedResponseBody<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

impl<T> UnitizedResponseBody<T> {
    pub fn success(data: T) -> UnitizedResponseBody<T> {
        UnitizedResponseBody {
            success: true,
            message: None,
            data: Some(data),
        }
    }

    pub fn failed(message: Option<String>) -> UnitizedResponseBody<T> {
        UnitizedResponseBody {
            success: false,
            message,
            data: None,
        }
    }
}
