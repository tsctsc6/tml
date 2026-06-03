use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the web API server
    Start,
    /// Manage admin account
    Manage {
        #[command(subcommand)]
        command: ManageCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ManageCommands {
    /// Create a new admin account
    InitAdmin {
        #[arg(short, long)]
        username: String,
    },
    /// Reset password for a new account
    ResetPassword {
        #[arg(short, long)]
        username: String,
    },
}
