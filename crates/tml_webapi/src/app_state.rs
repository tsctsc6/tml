use std::sync::Arc;

use moka::future::Cache;
use sea_orm::DatabaseConnection;
use tml_migration::prelude::uuid;

use crate::{command::Cli, config::AppConfig};

#[derive(Clone)]
pub struct AppState {
    pub app_config: Arc<AppConfig>,
    pub cli: Arc<Cli>,
    pub password_hasher: Arc<tml_infrastructure::password_hasher::PasswordHasher>,
    pub jwt_manager: Arc<tml_infrastructure::jwt_manager::JwtManager>,
    pub db: DatabaseConnection,
    pub user_id_security_stamp_cache: Cache<i64, uuid::Uuid>,
}
