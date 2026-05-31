use config::{Config, ConfigError, File as ConfigFile};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub connect_string: String,
    pub jwt_secret_key: String,
}

pub fn init_config() -> Result<AppConfig, ConfigError> {
    let builder = Config::builder().add_source(ConfigFile::with_name("config.toml"));
    let config = builder.build()?;
    config.try_deserialize::<AppConfig>()
}
