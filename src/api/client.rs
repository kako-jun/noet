use crate::auth::Credentials;
use crate::config::Config;
use crate::error::{NoetError, Result};
use reqwest::{Client, Response};
use std::time::Duration;

pub struct NoteClient {
    client: Client,
    base_url: String,
    config: Config,
    credentials: Credentials,
}

impl NoteClient {
    pub fn new(config: Config, credentials: Credentials) -> Result<Self> {
        let mut client_builder = Client::builder().timeout(Duration::from_secs(30));

        // Check for proxy environment variables
        if let Ok(http_proxy) = std::env::var("HTTP_PROXY") {
            if let Ok(proxy) = reqwest::Proxy::http(&http_proxy) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        if let Ok(https_proxy) = std::env::var("HTTPS_PROXY") {
            if let Ok(proxy) = reqwest::Proxy::https(&https_proxy) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder.build().map_err(NoetError::HttpError)?;
        let base_url = config.base_url.clone();

        Ok(Self {
            client,
            base_url,
            config,
            credentials,
        })
    }

    pub fn get_username(&self) -> Result<String> {
        self.config.username.clone().ok_or_else(|| {
            NoetError::ConfigError(
                "Username not configured. Please set username in config.toml".to_string(),
            )
        })
    }

    pub async fn get(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);

        let mut request = self
            .client
            .get(&url)
            .header("Cookie", &self.credentials.session_cookie);

        if let Some(ref csrf_token) = self.credentials.csrf_token {
            request = request.header("X-CSRF-Token", csrf_token);
        }

        let response = request.send().await.map_err(NoetError::HttpError)?;

        Self::check_response(response).await
    }

    pub async fn post(&self, path: &str, body: impl serde::Serialize) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);

        let mut request = self
            .client
            .post(&url)
            .header("Cookie", &self.credentials.session_cookie)
            .header("Content-Type", "application/json")
            .json(&body);

        if let Some(ref csrf_token) = self.credentials.csrf_token {
            request = request.header("X-CSRF-Token", csrf_token);
        }

        let response = request.send().await.map_err(NoetError::HttpError)?;

        Self::check_response(response).await
    }

    pub async fn put(&self, path: &str, body: impl serde::Serialize) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);

        let mut request = self
            .client
            .put(&url)
            .header("Cookie", &self.credentials.session_cookie)
            .header("Content-Type", "application/json")
            .json(&body);

        if let Some(ref csrf_token) = self.credentials.csrf_token {
            request = request.header("X-CSRF-Token", csrf_token);
        }

        let response = request.send().await.map_err(NoetError::HttpError)?;

        Self::check_response(response).await
    }

    pub async fn delete(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);

        let mut request = self
            .client
            .delete(&url)
            .header("Cookie", &self.credentials.session_cookie);

        if let Some(ref csrf_token) = self.credentials.csrf_token {
            request = request.header("X-CSRF-Token", csrf_token);
        }

        let response = request.send().await.map_err(NoetError::HttpError)?;

        Self::check_response(response).await
    }

    async fn check_response(response: Response) -> Result<Response> {
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let status_code = status.as_u16();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(NoetError::ApiError {
                status: status_code,
                message: error_text,
            })
        }
    }
}
