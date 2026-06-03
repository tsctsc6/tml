use tml_application::app_trait::jwt_manager::Trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request},
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub inner: tml_application::app_trait::jwt_manager::Claims,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value: &axum::http::HeaderValue| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let token = &auth_header[7..];

        let app_state = AppState::from_ref(state);

        let token_data = match app_state.jwt_manager.verify_token(token) {
            Ok(t) => t,
            Err(e) => {
                println!("{:?}", e);
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        Ok(Claims { inner: token_data })
    }
}
