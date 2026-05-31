pub mod app_state;
pub mod command;
pub mod config;

use std::{process::ExitCode, sync::Arc};

use clap::Parser;
use sea_orm::Database;
use tml_migration::MigratorTrait;

use crate::{app_state::AppState, command::Cli};

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
        cli,
        db,
    };

    match &app_state.cli.command {
        command::Commands::Start => start(),
        command::Commands::Manage { command } => match command {
            command::ManageCommands::Init { username } => init(username),
            command::ManageCommands::ResetPassword { username } => reset_password(username),
        },
    };

    ExitCode::SUCCESS
}

fn start() -> ! {
    loop {}
}

fn init(username: &str) {}

fn reset_password(username: &str) {}
