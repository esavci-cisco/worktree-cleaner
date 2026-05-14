// src/config.rs

use anyhow::{anyhow, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub roots: Vec<String>,
}

pub fn config_path() -> PathBuf {
    config_dir()
        .unwrap()
        .join("git-wt")
        .join("config.toml")
}

pub fn init_config() -> Result<()> {
    let path = config_path();

    if path.exists() {
        println!("Config already exists");
        return Ok(());
    }

    let parent = path.parent().unwrap();

    fs::create_dir_all(parent)?;

    let config = Config {
        roots: vec![],
    };

    fs::write(
        &path,
        toml::to_string_pretty(&config)?,
    )?;

    println!("Created config: {}", path.display());

    Ok(())
}

pub fn load_config() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        return Err(anyhow!(
            "Config not found. Run: git wt init"
        ));
    }

    let content = fs::read_to_string(path)?;

    Ok(toml::from_str(&content)?)
}
