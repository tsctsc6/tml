use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use tml_application::app_trait::jwt_manager::Claims;

pub struct JwtManager {
    secret: String,
}

impl JwtManager {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl tml_application::app_trait::jwt_manager::Trait for JwtManager {
    /// Create jwt.
    /// claims.exp is no need to set, this method will set it.
    /// claims will be destroyed after calling the method.
    fn create_token(
        &self,
        mut claims: Claims,
        exp: Duration,
    ) -> Result<String, tml_application::app_trait::jwt_manager::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            + exp.as_millis();
        claims.exp = timestamp;
        let token_str = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .map_err(|e| tml_application::app_trait::jwt_manager::Error::JwtError(e.to_string()))?;
        Ok(token_str)
    }

    fn verify_token(
        &self,
        token: &str,
    ) -> Result<Claims, tml_application::app_trait::jwt_manager::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| tml_application::app_trait::jwt_manager::Error::JwtError(e.to_string()))?;
        Ok(token_data.claims)
    }
}
