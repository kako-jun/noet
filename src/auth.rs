use crate::error::{NoetError, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "noet";
const COOKIE_KEY: &str = "note_session_cookie";
const CSRF_TOKEN_KEY: &str = "note_csrf_token";

/// Helper function to get keyring entry for session cookie
fn get_cookie_entry() -> Result<Entry> {
    Entry::new(SERVICE_NAME, COOKIE_KEY).map_err(NoetError::KeyringError)
}

/// Helper function to get keyring entry for CSRF token
fn get_csrf_entry() -> Result<Entry> {
    Entry::new(SERVICE_NAME, CSRF_TOKEN_KEY).map_err(NoetError::KeyringError)
}

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
        let cookie_entry = get_cookie_entry()?;
        cookie_entry
            .set_password(&self.session_cookie)
            .map_err(NoetError::KeyringError)?;

        if let Some(ref csrf_token) = self.csrf_token {
            let csrf_entry = get_csrf_entry()?;
            csrf_entry
                .set_password(csrf_token)
                .map_err(NoetError::KeyringError)?;
        }

        Ok(())
    }

    /// Load credentials from system keyring
    pub fn load() -> Result<Self> {
        let cookie_entry = get_cookie_entry()?;
        let session_cookie = cookie_entry.get_password().map_err(|e| {
            NoetError::AuthError(format!(
                "Not authenticated. Please run 'noet auth login' first. Error: {}",
                e
            ))
        })?;

        let csrf_token = get_csrf_entry()
            .and_then(|entry| entry.get_password().map_err(NoetError::KeyringError))
            .ok();

        Ok(Self {
            session_cookie,
            csrf_token,
        })
    }

    /// Check if credentials exist
    pub fn exists() -> bool {
        get_cookie_entry()
            .and_then(|entry| entry.get_password().map_err(NoetError::KeyringError))
            .is_ok()
    }

    /// Delete credentials from system keyring
    pub fn delete() -> Result<()> {
        let cookie_entry = get_cookie_entry()?;
        cookie_entry
            .delete_credential()
            .map_err(NoetError::KeyringError)?;

        // Try to delete CSRF token, but don't fail if it doesn't exist
        let _ = get_csrf_entry()
            .and_then(|entry| entry.delete_credential().map_err(NoetError::KeyringError));

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
