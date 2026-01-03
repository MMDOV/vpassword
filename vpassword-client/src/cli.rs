use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "password", about = "A simple password manager CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init { vault_path: Option<PathBuf> },
    Open { vault_path: Option<PathBuf> },
    Close,
    Generate { name: String, username: String },
    Add { name: String, username: String },
    Show { name: String },
    List,
    Remove { name: String },
}

pub fn parse_cli() -> Commands {
    Cli::parse().command
}
