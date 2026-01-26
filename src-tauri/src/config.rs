use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use dirs;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub staging_dir: Option<String>,
    pub drive_folder: Option<String>,
    pub sync_interval: Option<u64>, // minutes
    pub auto_sync: Option<bool>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            staging_dir: None,
            drive_folder: Some("sync-bot-backups".to_string()),
            sync_interval: Some(60),
            auto_sync: Some(false),
            client_id: None,
            client_secret: None,
        }
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("sync-bot");
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
    }
    
    Ok(config_dir)
}

pub fn get_data_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .context("Failed to get data directory")?
        .join("sync-bot");
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;
    }
    
    Ok(data_dir)
}

pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.toml"))
}

pub fn get_staging_dir() -> Result<PathBuf> {
    // Check environment variable first
    if let Ok(env_path) = std::env::var("SYNC_BOT_STAGING_DIR") {
        let path = PathBuf::from(env_path);
        if !path.exists() {
            fs::create_dir_all(&path)
                .context("Failed to create staging directory from env")?;
        }
        return Ok(path);
    }

    // Check config file
    let config = load_config()?;
    if let Some(ref staging_dir) = config.staging_dir {
        let path = PathBuf::from(staging_dir);
        if !path.exists() {
            fs::create_dir_all(&path)
                .context("Failed to create staging directory from config")?;
        }
        return Ok(path);
    }

    // Default to data directory
    let default_dir = get_data_dir()?.join("staging");
    if !default_dir.exists() {
        fs::create_dir_all(&default_dir)
            .context("Failed to create default staging directory")?;
    }
    Ok(default_dir)
}

pub fn get_archives_dir() -> Result<PathBuf> {
    let archives_dir = get_data_dir()?.join("archives");
    if !archives_dir.exists() {
        fs::create_dir_all(&archives_dir)
            .context("Failed to create archives directory")?;
    }
    Ok(archives_dir)
}

pub fn init_config() -> Result<()> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        let default_config = Config::default();
        save_config(&default_config)?;
    }
    
    Ok(())
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Ok(Config::default());
    }
    
    let content = fs::read_to_string(&config_path)
        .context("Failed to read config file")?;
    
    let config: Config = toml::from_str(&content)
        .context("Failed to parse config file")?;
    
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(config)
        .context("Failed to serialize config")?;
    
    fs::write(&config_path, content)
        .context("Failed to write config file")?;
    
    Ok(())
}

pub fn update_config<F>(updater: F) -> Result<Config>
where
    F: FnOnce(&mut Config),
{
    let mut config = load_config()?;
    updater(&mut config);
    save_config(&config)?;
    Ok(config)
}
