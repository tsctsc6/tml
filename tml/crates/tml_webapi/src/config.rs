use config::{Config, ConfigError, File as ConfigFile};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub listening_address: String,
    pub log_level: String,
    pub scan_extensions: Vec<String>,
    pub database: Database,
    pub jwt: Jwt,
    pub user_id_security_stamp_cache: UserIdSecurityStampCache,
    pub meilisearch: Meilisearch,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub connection_string: String,
}

#[derive(Debug, Deserialize)]
pub struct Jwt {
    pub secret: String,
    pub exp_in_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct UserIdSecurityStampCache {
    pub max_capacity: u64,
    pub time_to_live_in_second: u64,
}

#[derive(Debug, Deserialize)]
pub struct Meilisearch {
    pub host: String,
    pub api_key: String,
    pub index_name: String,
}

pub fn init_config() -> Result<AppConfig, ConfigError> {
    let builder = Config::builder().add_source(ConfigFile::with_name("config.toml"));
    let config = builder.build()?;
    config.try_deserialize::<AppConfig>()
}
