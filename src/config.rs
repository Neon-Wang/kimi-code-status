use crate::types::AppConfig;
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("kimi-code-status")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if !path.exists() {
        return AppConfig::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|e| {
                log::warn!("Failed to parse config, using defaults: {e}");
                AppConfig::default()
            })
        }
        Err(e) => {
            log::warn!("Failed to read config file, using defaults: {e}");
            AppConfig::default()
        }
    }
}

pub fn save_config(config: &AppConfig) {
    let dir = config_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        log::error!("Failed to create config directory: {e}");
        return;
    }

    match serde_json::to_string_pretty(config) {
        Ok(content) => {
            if let Err(e) = std::fs::write(config_path(), content) {
                log::error!("Failed to write config: {e}");
            }
        }
        Err(e) => {
            log::error!("Failed to serialize config: {e}");
        }
    }
}
