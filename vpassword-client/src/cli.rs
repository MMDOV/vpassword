use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "password", about = "A simple password manager CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Vault {
        #[command(subcommand)]
        command: VaultCommands,
    },
    Password {
        #[command(subcommand)]
        command: PasswordCommands,
    },
}

/// Vault related actions
#[derive(Subcommand, Debug)]
pub enum VaultCommands {
    /// Create a new vault
    Create {
        vault_name: String,
        master_pass: String,
    },
    /// Remove an existing vault
    Remove {
        vault_name: String,
        master_pass: String,
    },
    /// Returns a list of all the vaults
    List {
        vault_name: String,
        master_pass: String,
    },
}

#[derive(Subcommand, Debug)]
/// Password related actions
pub enum PasswordCommands {
    /// Add a new password entry
    Add {
        vault_name: String,
        master_pass: String,
        name: String,
        username: String,
        /// can enter "generate" for a password to be generated automatically
        password: String,
    },
    /// Remove a password entry
    Remove {
        vault_name: String,
        master_pass: String,
        name: String,
    },
    /// Generate a safe password
    Generate {},
}

pub fn parse_cli() -> Commands {
    Cli::parse().command
}
