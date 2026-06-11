pub mod app_state;
pub mod command;
pub mod config;
pub mod endpoint;
pub mod extractor;
pub mod logger;
pub mod manage;

use std::{process::ExitCode, sync::Arc, time::Duration};

use axum::routing::{get, post};
use clap::Parser;
use moka::future::Cache;
use sea_orm::Database;
use tml_migration::MigratorTrait;

use crate::{app_state::AppState, command::Cli};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Arc::new(Cli::parse());

    let app_config = match config::init_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };
    let app_config = Arc::new(app_config);

    match logger::init_logger(&app_config.log_level) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    let db = match Database::connect(&app_config.database.connection_string).await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("{}", e.to_string());
            return ExitCode::FAILURE;
        }
    };
    match tml_migration::Migrator::up(&db, None).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{}", e.to_string());
            return ExitCode::FAILURE;
        }
    };

    let user_id_security_stamp_cache = Cache::builder()
        .max_capacity(app_config.user_id_security_stamp_cache.max_capacity)
        .time_to_live(Duration::from_secs(
            app_config
                .user_id_security_stamp_cache
                .time_to_live_in_second,
        ))
        .build();

    let app_state = AppState {
        app_config: Arc::clone(&app_config),
        cli: Arc::clone(&cli),
        password_hasher: tml_infrastructure::password_hasher::PasswordHasher,
        jwt_manager: tml_infrastructure::jwt_manager::JwtManager::new(
            &app_config.jwt.secret,
            app_config.jwt.exp_in_seconds,
        ),
        db,
        user_id_security_stamp_cache,
        music_info_provider: tml_infrastructure::music_info_provider::MusicInfoProvider,
    };

    let result = match &Arc::clone(&cli).command {
        command::Commands::Start => start(app_state).await,
        command::Commands::Manage { command } => manage(command, app_state).await,
    };

    return result;
}

async fn start(app_state: AppState) -> ExitCode {
    let listening_address = app_state.app_config.listening_address.as_str();
    let listener = match tokio::net::TcpListener::bind(listening_address).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("{}", e.to_string());
            return ExitCode::FAILURE;
        }
    };
    let user_routes = axum::Router::new()
        .route("/register", post(endpoint::auth::register::handle))
        .route("/login", post(endpoint::auth::login::handle));
    let mgmt_routes = axum::Router::new()
        .route("/create_job", post(endpoint::mgmt::create_job::handle))
        .route(
            "/create_storage",
            post(endpoint::mgmt::create_storage::handle),
        )
        .route(
            "/update_storage",
            post(endpoint::mgmt::update_storage::handle),
        )
        .route("/delete_job", post(endpoint::mgmt::delete_job::handle))
        .route(
            "/delete_storage",
            post(endpoint::mgmt::delete_storage::handle),
        )
        .route(
            "/read_all_storage",
            get(endpoint::mgmt::read_all_storage::handle),
        );
    let app = axum::Router::new()
        .nest("/api/mgmt", mgmt_routes)
        .nest("/api/user", user_routes)
        .with_state(app_state);
    match axum::serve(listener, app).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{}", e.to_string());
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}

async fn manage(command: &command::ManageCommands, app_state: AppState) -> ExitCode {
    let result = match command {
        command::ManageCommands::InitAdmin { username } => {
            manage::init_admin(username, app_state).await
        }
        command::ManageCommands::ResetPassword { username } => {
            manage::reset_password(username, app_state).await
        }
    };
    match result {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{}", e.to_string());
        }
    }
    ExitCode::SUCCESS
}
