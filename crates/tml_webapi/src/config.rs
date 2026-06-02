use config::{Config, ConfigError, File as ConfigFile};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub connect_string: String,
    pub jwt_secret_key: String,
    pub listening_address: String,
    pub user_id_security_stamp_cache: UserIdSecurityStampCache,
}

#[derive(Debug, Deserialize)]
pub struct UserIdSecurityStampCache {
    pub max_capacity: u64,
    pub time_to_live_in_second: u64,
}

pub fn init_config() -> Result<AppConfig, ConfigError> {
    let builder = Config::builder().add_source(ConfigFile::with_name("config.toml"));
    let config = builder.build()?;
    config.try_deserialize::<AppConfig>()
}
