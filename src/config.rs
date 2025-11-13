use crate::error::{NoetError, Result};
use crate::workspace;
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

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
            username: None,
            base_url: default_base_url(),
        }
    }
}

impl Config {
    /// Load configuration with workspace config taking precedence over global config
    pub fn load() -> Result<Self> {
        // Load global config
        let mut config = Self::load_global()?;

        // Try to load workspace config and merge
        if let Ok(workspace_config) = Self::load_workspace() {
            config = config.merge(workspace_config);
        }

        Ok(config)
    }

    /// Load global configuration only
    fn load_global() -> Result<Self> {
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

    /// Load workspace configuration (.noet/config.toml)
    fn load_workspace() -> Result<Self> {
        let workspace_root = workspace::find_workspace_root()?;
        let workspace_config_path = workspace_root.join(".noet").join(CONFIG_FILE);

        if !workspace_config_path.exists() {
            return Err(NoetError::ConfigError(
                "Workspace config not found".to_string(),
            ));
        }

        let content = fs::read_to_string(&workspace_config_path).map_err(|e| {
            NoetError::ConfigError(format!("Failed to read workspace config: {}", e))
        })?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Merge two configs, with other taking precedence
    fn merge(self, other: Self) -> Self {
        Self {
            default_status: other.default_status.or(self.default_status),
            default_tags: other.default_tags.or(self.default_tags),
            editor: other.editor.or(self.editor),
            username: other.username.or(self.username),
            base_url: if other.base_url != default_base_url() {
                other.base_url
            } else {
                self.base_url
            },
        }
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

    #[test]
    fn test_config_merge_workspace_takes_precedence() {
        let global = Config {
            default_status: Some("draft".to_string()),
            default_tags: Some(vec!["global".to_string()]),
            editor: Some("vim".to_string()),
            username: Some("global-user".to_string()),
            base_url: "https://note.com".to_string(),
        };

        let workspace = Config {
            default_status: Some("published".to_string()),
            default_tags: None,
            editor: None,
            username: Some("workspace-user".to_string()),
            base_url: "https://note.com".to_string(),
        };

        let merged = global.merge(workspace);

        assert_eq!(merged.default_status, Some("published".to_string()));
        assert_eq!(merged.default_tags, Some(vec!["global".to_string()]));
        assert_eq!(merged.editor, Some("vim".to_string()));
        assert_eq!(merged.username, Some("workspace-user".to_string()));
    }

    #[test]
    fn test_config_merge_preserves_global_when_workspace_empty() {
        let global = Config {
            default_status: Some("draft".to_string()),
            default_tags: Some(vec!["tag1".to_string()]),
            editor: Some("code".to_string()),
            username: Some("user".to_string()),
            base_url: "https://note.com".to_string(),
        };

        let workspace = Config {
            default_status: None,
            default_tags: None,
            editor: None,
            username: None,
            base_url: "https://note.com".to_string(),
        };

        let merged = global.merge(workspace);

        assert_eq!(merged.default_status, Some("draft".to_string()));
        assert_eq!(merged.default_tags, Some(vec!["tag1".to_string()]));
        assert_eq!(merged.editor, Some("code".to_string()));
        assert_eq!(merged.username, Some("user".to_string()));
    }
}
