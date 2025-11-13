use thiserror::Error;

#[derive(Error, Debug)]
pub enum NoetError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("Dialoguer error: {0}")]
    DialoguerError(#[from] dialoguer::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid input: {0}")]
    #[allow(dead_code)]
    InvalidInput(String),

    #[error("Article not found: {0}")]
    ArticleNotFound(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Unknown error: {0}")]
    #[allow(dead_code)]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, NoetError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let error = NoetError::AuthError("Test auth error".to_string());
        assert_eq!(error.to_string(), "Authentication failed: Test auth error");
    }

    #[test]
    fn test_api_error_display() {
        let error = NoetError::ApiError {
            status: 404,
            message: "Not found".to_string(),
        };
        assert_eq!(error.to_string(), "API error: 404 - Not found");
    }

    #[test]
    fn test_config_error_display() {
        let error = NoetError::ConfigError("Config file missing".to_string());
        assert_eq!(
            error.to_string(),
            "Configuration error: Config file missing"
        );
    }

    #[test]
    fn test_file_not_found_error_display() {
        let error = NoetError::FileNotFound("/path/to/file.md".to_string());
        assert_eq!(error.to_string(), "File not found: /path/to/file.md");
    }

    #[test]
    fn test_article_not_found_error_display() {
        let error = NoetError::ArticleNotFound("article-123".to_string());
        assert_eq!(error.to_string(), "Article not found: article-123");
    }

    #[test]
    fn test_missing_field_error_display() {
        let error = NoetError::MissingField("title".to_string());
        assert_eq!(error.to_string(), "Missing required field: title");
    }
}
