pub mod app_state;
pub mod command;
pub mod config;
pub mod manage;

use std::{process::ExitCode, sync::Arc};

use clap::Parser;
use sea_orm::Database;
use tml_migration::MigratorTrait;

use crate::{app_state::AppState, command::Cli};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Webapi error")]
    WebapiError,
    #[error("{0}")]
    ManageError(#[from] manage::Error),
}

#[tokio::main]
async fn main() -> ExitCode {
    let app_config = match config::init_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return ExitCode::FAILURE;
        }
    };
    let app_config = Arc::new(app_config);
    let db = match Database::connect(&app_config.connect_string).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return ExitCode::FAILURE;
        }
    };
    match tml_migration::Migrator::up(&db, None).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e.to_string());
            return ExitCode::FAILURE;
        }
    };
    let cli = Arc::new(Cli::parse());

    let app_state = AppState {
        app_config,
        cli: cli.clone(),
        db,
    };

    let result = match &cli.clone().command {
        command::Commands::Start => start(app_state),
        command::Commands::Manage { command } => manage(command, app_state).await,
    };
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e.to_string());
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}

fn start(app_state: AppState) -> Result<(), Error> {
    Ok(())
}

async fn manage(command: &command::ManageCommands, app_state: AppState) -> Result<(), Error> {
    let _x = match command {
        command::ManageCommands::InitAdmin { username } => manage::init(username, app_state).await,
        command::ManageCommands::ResetPassword { username } => {
            manage::reset_password(username, app_state).await
        }
    }?;
    Ok(())
}
