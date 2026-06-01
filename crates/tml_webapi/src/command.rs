use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Increase verbosity, repeat for more verbosity, default is 3 (info)
    #[arg(
        short = 'v',
        long,
        action = clap::ArgAction::Count,
        global = true,
        default_value_t = 3
    )]
    pub verbose: u8,
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
