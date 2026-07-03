use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request},
};
use sea_orm::{DbErr, EntityTrait};
use serde::{Deserialize, Serialize};
use tml_application::app_trait::jwt_manager::Trait;
use tml_infrastructure::entity::auth::user;

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
                tracing::warn!("{:?}", e);
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        let db: sea_orm::prelude::DatabaseConnection = app_state.db.clone();

        let security_stamp = match app_state
            .user_id_security_stamp_cache
            .try_get_with::<_, DbErr>(token_data.sub, async {
                tracing::info!("Query security_stamp from the database");
                let user = user::Entity::find_by_id(token_data.sub).one(&db).await?;
                let x = user.map(|u| u.security_stamp);
                Ok(x)
            })
            .await
        {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("{:?}", e);
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        let security_stamp = match security_stamp {
            Some(s) => s,
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        if security_stamp != token_data.security_stamp {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(Claims { inner: token_data })
    }
}
