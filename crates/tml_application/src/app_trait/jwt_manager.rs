use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    JwtError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: u128,
    pub roles: Vec<String>,
    pub security_stamp: uuid::Uuid,
}

pub trait Trait: Send + Sync + 'static {
    /// claims.exp will be set in method
    fn create_token(&self, claims: Claims, exp: Duration) -> Result<String, Error>;
    fn verify_token(&self, token: &str) -> Result<Claims, Error>;
}
