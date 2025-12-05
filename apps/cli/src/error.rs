use thiserror::Error;

#[derive(Error, Debug)]
pub enum NoetError {
    #[error("ネットワークエラー: {0}")]
    Network(String),

    #[error("設定エラー: {0}")]
    ConfigError(String),

    #[error("IOエラー: {0}")]
    IoError(#[from] std::io::Error),

    #[error("シリアライズエラー: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("対話型入力エラー: {0}")]
    DialoguerError(#[from] dialoguer::Error),

    #[error("ファイルが見つかりません: {0}")]
    FileNotFound(String),

    #[error("拡張機能エラー: {0}")]
    Extension(String),
}

pub type Result<T> = std::result::Result<T, NoetError>;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_extension_error_display() {
        let error = NoetError::Extension("接続に失敗しました".to_string());
        assert_eq!(error.to_string(), "拡張機能エラー: 接続に失敗しました");
    }
}
