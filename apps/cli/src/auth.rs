use crate::error::{NoetError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub session_cookie: String,
    pub xsrf_token: Option<String>,
}

impl Credentials {
    #[allow(dead_code)]
    pub fn new(session_cookie: String, xsrf_token: Option<String>) -> Self {
        Self {
            session_cookie,
            xsrf_token,
        }
    }

    /// Load credentials from environment variables
    pub fn load() -> Result<Self> {
        let session_cookie = std::env::var("NOET_SESSION_COOKIE").map_err(|_| {
            NoetError::AuthError(
                "認証されていません。環境変数 NOET_SESSION_COOKIE を設定してください。".to_string(),
            )
        })?;

        let xsrf_token = std::env::var("NOET_XSRF_TOKEN").ok();

        Ok(Self {
            session_cookie,
            xsrf_token,
        })
    }

    /// Check if credentials exist in environment
    pub fn exists() -> bool {
        std::env::var("NOET_SESSION_COOKIE").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_creation() {
        let creds = Credentials::new(
            "_note_session_v5=test_session".to_string(),
            Some("test_xsrf_token".to_string()),
        );

        assert_eq!(creds.session_cookie, "_note_session_v5=test_session");
        assert_eq!(creds.xsrf_token, Some("test_xsrf_token".to_string()));
    }
}
