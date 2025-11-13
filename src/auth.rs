use crate::error::{NoetError, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "noet";
const COOKIE_KEY: &str = "note_session_cookie";
const CSRF_TOKEN_KEY: &str = "note_csrf_token";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub session_cookie: String,
    pub csrf_token: Option<String>,
}

impl Credentials {
    pub fn new(session_cookie: String, csrf_token: Option<String>) -> Self {
        Self {
            session_cookie,
            csrf_token,
        }
    }

    /// Save credentials to system keyring
    pub fn save(&self) -> Result<()> {
        let cookie_entry = Entry::new(SERVICE_NAME, COOKIE_KEY)
            .map_err(|e| NoetError::KeyringError(e))?;

        cookie_entry
            .set_password(&self.session_cookie)
            .map_err(|e| NoetError::KeyringError(e))?;

        if let Some(ref csrf_token) = self.csrf_token {
            let csrf_entry = Entry::new(SERVICE_NAME, CSRF_TOKEN_KEY)
                .map_err(|e| NoetError::KeyringError(e))?;

            csrf_entry
                .set_password(csrf_token)
                .map_err(|e| NoetError::KeyringError(e))?;
        }

        Ok(())
    }

    /// Load credentials from system keyring
    pub fn load() -> Result<Self> {
        let cookie_entry = Entry::new(SERVICE_NAME, COOKIE_KEY)
            .map_err(|e| NoetError::KeyringError(e))?;

        let session_cookie = cookie_entry
            .get_password()
            .map_err(|e| NoetError::AuthError(format!("Not authenticated. Please run 'noet auth login' first. Error: {}", e)))?;

        let csrf_token = Entry::new(SERVICE_NAME, CSRF_TOKEN_KEY)
            .and_then(|entry| entry.get_password())
            .ok();

        Ok(Self {
            session_cookie,
            csrf_token,
        })
    }

    /// Check if credentials exist
    pub fn exists() -> bool {
        Entry::new(SERVICE_NAME, COOKIE_KEY)
            .and_then(|entry| entry.get_password())
            .is_ok()
    }

    /// Delete credentials from system keyring
    pub fn delete() -> Result<()> {
        let cookie_entry = Entry::new(SERVICE_NAME, COOKIE_KEY)
            .map_err(|e| NoetError::KeyringError(e))?;

        cookie_entry
            .delete_credential()
            .map_err(|e| NoetError::KeyringError(e))?;

        // Try to delete CSRF token, but don't fail if it doesn't exist
        let _ = Entry::new(SERVICE_NAME, CSRF_TOKEN_KEY)
            .and_then(|entry| entry.delete_credential());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_creation() {
        let creds = Credentials::new(
            "_note_session_v5=test_session".to_string(),
            Some("test_csrf_token".to_string()),
        );

        assert_eq!(creds.session_cookie, "_note_session_v5=test_session");
        assert_eq!(creds.csrf_token, Some("test_csrf_token".to_string()));
    }
}
