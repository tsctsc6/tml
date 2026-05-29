use config::{Config, File as ConfigFile};
use sea_orm::Database;
use serde::Deserialize;
use tml_migration::MigratorTrait;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub connect_string: String,
    pub jwt_secret_key: String,
}

#[tokio::main]
async fn main() {
    let builder = Config::builder().add_source(ConfigFile::with_name("config.toml"));
    let config = builder.build().unwrap();
    let app_config: AppConfig = config.try_deserialize::<AppConfig>().unwrap();

    let db = Database::connect(&app_config.connect_string).await.unwrap();
    tml_migration::Migrator::up(&db, None).await.unwrap();
}
