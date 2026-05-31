use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{command::Cli, config::AppConfig};

#[derive(Debug, Clone)]
pub struct AppState {
    pub app_config: Arc<AppConfig>,
    pub cli: Arc<Cli>,
    pub db: DatabaseConnection,
}
