use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AuthConfig {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: u64,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub auth: Option<AuthConfig>,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
}

pub fn get_config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("GEMINI_PROXY_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    
    let mut path = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".gemini-proxy");
    path
}

pub fn get_config_file() -> PathBuf {
    let mut path = get_config_dir();
    path.push("config.json");
    path
}

pub fn load_config() -> Result<Config> {
    let path = get_config_file();
    if !path.exists() {
        return Ok(Config::default());
    }
    
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {:?}", path))?;
    
    serde_json::from_str(&content)
        .with_context(|| "Failed to parse config JSON")
}

pub fn save_config(config: &Config) -> Result<()> {
    let dir = get_config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory at {:?}", dir))?;
    }
    
    let path = get_config_file();
    let content = serde_json::to_string_pretty(config)
        .with_context(|| "Failed to serialize config to JSON")?;
    
    fs::write(&path, content)
        .with_context(|| format!("Failed to write config file at {:?}", path))?;
    
    Ok(())
}
