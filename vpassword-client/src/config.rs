use std::io::prelude::*;
use std::{fs::File, path::PathBuf};

use crate::models::AppConfig;

pub fn get_config_path() -> PathBuf {
    let mut path = match dirs::config_dir() {
        Some(p) => p,
        None => dirs::home_dir().expect("Could not determine home directory"),
    };

    path.push("vpassword");

    path
}

pub fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_path = get_config_path();
    std::fs::create_dir_all(&config_path)?;
    let config_string = toml::to_string(config)?;
    config_path.push("config.toml");
    let mut file = File::create(config_path)?;

    file.write_all(config_string.as_ref())?;
    Ok(())
}
pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let mut config_path = get_config_path();
    config_path.push("config.toml");
    if config_path.exists() {
        let config: AppConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
        Ok(config)
    } else {
        let config: AppConfig = AppConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}
