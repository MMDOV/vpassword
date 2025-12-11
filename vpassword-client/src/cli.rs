use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "password", about = "A simple password manager CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init { vault_name: String },
    Open { vault_name: String },
    Close { vault_name: String },
    Generate { name: String, username: String },
    Show { name: String },
    List,
    Delete { name: String },
}

pub fn parse_cli() -> Commands {
    Cli::parse().command
}
