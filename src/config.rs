use crate::error::{NoetError, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "noet";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,

    #[serde(default = "default_base_url")]
    pub base_url: String,
}

fn default_base_url() -> String {
    "https://note.com".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_status: Some("draft".to_string()),
            default_tags: None,
            editor: None,
            base_url: default_base_url(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| NoetError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                NoetError::ConfigError(format!("Failed to create config directory: {}", e))
            })?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| NoetError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, content)
            .map_err(|e| NoetError::ConfigError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = config_dir().ok_or_else(|| {
            NoetError::ConfigError("Could not determine config directory".to_string())
        })?;

        Ok(config_dir.join(APP_NAME).join(CONFIG_FILE))
    }

    #[allow(dead_code)]
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = config_dir().ok_or_else(|| {
            NoetError::ConfigError("Could not determine config directory".to_string())
        })?;

        Ok(config_dir.join(APP_NAME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.base_url, "https://note.com");
        assert_eq!(config.default_status, Some("draft".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.base_url, deserialized.base_url);
    }
}
