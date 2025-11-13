use thiserror::Error;

#[derive(Error, Debug)]
pub enum NoetError {
    #[error("HTTPリクエストに失敗しました: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("認証に失敗しました: {0}")]
    AuthError(String),

    #[error("APIエラー: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("設定エラー: {0}")]
    ConfigError(String),

    #[error("IOエラー: {0}")]
    IoError(#[from] std::io::Error),

    #[error("シリアライズエラー: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("TOML解析エラー: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("キーリングエラー: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("対話型入力エラー: {0}")]
    DialoguerError(#[from] dialoguer::Error),

    #[error("ファイルが見つかりません: {0}")]
    FileNotFound(String),

    #[error("無効な入力: {0}")]
    #[allow(dead_code)]
    InvalidInput(String),

    #[error("記事が見つかりません: {0}")]
    ArticleNotFound(String),

    #[error("必須フィールドがありません: {0}")]
    MissingField(String),

    #[error("不明なエラー: {0}")]
    #[allow(dead_code)]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, NoetError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let error = NoetError::AuthError("テスト認証エラー".to_string());
        assert_eq!(error.to_string(), "認証に失敗しました: テスト認証エラー");
    }

    #[test]
    fn test_api_error_display() {
        let error = NoetError::ApiError {
            status: 404,
            message: "見つかりません".to_string(),
        };
        assert_eq!(error.to_string(), "APIエラー: 404 - 見つかりません");
    }

    #[test]
    fn test_config_error_display() {
        let error = NoetError::ConfigError("設定ファイルが見つかりません".to_string());
        assert_eq!(
            error.to_string(),
            "設定エラー: 設定ファイルが見つかりません"
        );
    }

    #[test]
    fn test_file_not_found_error_display() {
        let error = NoetError::FileNotFound("/path/to/file.md".to_string());
        assert_eq!(
            error.to_string(),
            "ファイルが見つかりません: /path/to/file.md"
        );
    }

    #[test]
    fn test_article_not_found_error_display() {
        let error = NoetError::ArticleNotFound("article-123".to_string());
        assert_eq!(error.to_string(), "記事が見つかりません: article-123");
    }

    #[test]
    fn test_missing_field_error_display() {
        let error = NoetError::MissingField("title".to_string());
        assert_eq!(error.to_string(), "必須フィールドがありません: title");
    }
}
