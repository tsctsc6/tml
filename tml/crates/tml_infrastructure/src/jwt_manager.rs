use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use tml_application::app_trait::jwt_manager::Claims;

#[derive(Clone)]
pub struct JwtManager {
    config: Arc<Config>,
}

pub struct Config {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    exp_in_seconds: u64,
}

impl JwtManager {
    pub fn try_new(secret: impl Into<String>, exp_in_seconds: u64) -> Result<Self, tml_application::app_trait::jwt_manager::Error> {
        let secret = secret.into();
        let secret_bytes = secret.as_ref();
        let encoding_key = EncodingKey::from_secret(secret_bytes)
            .map_err(|e| tml_application::app_trait::jwt_manager::Error::JwtError(e.to_string()))?;
        let decoding_key = DecodingKey::from_secret(secret_bytes)
            .map_err(|e| tml_application::app_trait::jwt_manager::Error::JwtError(e.to_string()))?;

        Ok(Self {
            config: Arc::new(Config {
                encoding_key,
                decoding_key,
                exp_in_seconds,
            }),
        })
    }
}

impl tml_application::app_trait::jwt_manager::Trait for JwtManager {
    /// Create jwt.
    /// claims.exp is no need to set, this method will set it.
    /// claims will be destroyed after calling the method.
    fn create_token(
        &self,
        mut claims: Claims,
    ) -> Result<String, tml_application::app_trait::jwt_manager::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| tml_application::app_trait::jwt_manager::Error::JwtError(e.to_string()))?
            .as_millis()
            + Duration::from_secs(self.config.exp_in_seconds).as_millis();
        claims.exp = timestamp;
        let token_str = encode(
            &Header::default(),
            &claims,
            &self.config.encoding_key,
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
            &self.config.decoding_key,
            &Validation::default(),
        )
        .map_err(|e| tml_application::app_trait::jwt_manager::Error::JwtError(e.to_string()))?;
        Ok(token_data.claims)
    }
}
