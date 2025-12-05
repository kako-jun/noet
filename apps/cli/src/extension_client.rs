//! Extension client for communicating with browser extension via WebSocket
//!
//! The CLI starts a WebSocket server on localhost:9876
//! The browser extension connects to it and executes commands

use crate::error::{NoetError, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::timeout;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use uuid::Uuid;

const WEBSOCKET_PORT: u16 = 9876;
const COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

/// Request sent to extension
#[derive(Debug, Clone, Serialize)]
pub struct ExtensionRequest {
    pub id: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Response from extension
#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<ExtensionError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionError {
    pub code: String,
    pub message: String,
}

/// Article data from extension
#[derive(Debug, Clone, Deserialize)]
pub struct ArticleData {
    pub key: Option<String>,
    pub title: String,
    pub html: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub date: Option<String>,
    pub published_at: Option<String>,
}

/// Article list response
#[derive(Debug, Clone, Deserialize)]
pub struct ArticleListData {
    pub articles: Vec<ArticleData>,
    pub count: usize,
}

/// Auth status response
#[derive(Debug, Clone, Deserialize)]
pub struct AuthStatusData {
    pub logged_in: bool,
    pub username: Option<String>,
}

/// Pending request waiting for response
type PendingRequest = oneshot::Sender<ExtensionResponse>;

/// Extension client that communicates via WebSocket
pub struct ExtensionClient {
    /// Sender for outgoing messages
    tx: mpsc::Sender<String>,
    /// Pending requests waiting for responses
    pending: Arc<Mutex<HashMap<String, PendingRequest>>>,
}

impl ExtensionClient {
    /// Connect to a running extension server or start waiting for connection
    pub async fn connect() -> Result<Self> {
        let listener = TcpListener::bind(format!("127.0.0.1:{WEBSOCKET_PORT}"))
            .await
            .map_err(|e| NoetError::Network(format!("Failed to bind WebSocket server: {e}")))?;

        println!("WebSocket サーバーを起動しました (ws://127.0.0.1:{WEBSOCKET_PORT})");
        println!("ブラウザ拡張機能からの接続を待っています...");

        // Wait for connection with timeout
        let (stream, addr) = timeout(Duration::from_secs(30), listener.accept())
            .await
            .map_err(|_| {
                NoetError::Network("拡張機能からの接続がタイムアウトしました。拡張機能がインストールされていることを確認してください。".into())
            })?
            .map_err(|e| NoetError::Network(format!("Failed to accept connection: {e}")))?;

        println!("拡張機能が接続しました: {addr}");

        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| NoetError::Network(format!("WebSocket handshake failed: {e}")))?;

        Self::from_stream(ws_stream).await
    }

    /// Create client from existing WebSocket stream
    async fn from_stream(ws_stream: WebSocketStream<TcpStream>) -> Result<Self> {
        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<String>(32);
        let pending: Arc<Mutex<HashMap<String, PendingRequest>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();

        // Spawn writer task
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        // Spawn reader task
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(response) = serde_json::from_str::<ExtensionResponse>(&text) {
                            let mut pending = pending_clone.lock().await;
                            if let Some(sender) = pending.remove(&response.id) {
                                let _ = sender.send(response);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
        });

        Ok(Self { tx, pending })
    }

    /// Send a command to the extension and wait for response
    async fn send_command(
        &self,
        command: &str,
        params: Option<serde_json::Value>,
    ) -> Result<ExtensionResponse> {
        let id = Uuid::new_v4().to_string();

        let request = ExtensionRequest {
            id: id.clone(),
            command: command.to_string(),
            params,
        };

        let (response_tx, response_rx) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id.clone(), response_tx);
        }

        // Send request
        let json = serde_json::to_string(&request)?;
        self.tx
            .send(json)
            .await
            .map_err(|e| NoetError::Network(format!("Failed to send message: {e}")))?;

        // Wait for response with timeout
        let response = timeout(COMMAND_TIMEOUT, response_rx)
            .await
            .map_err(|_| NoetError::Network("コマンドがタイムアウトしました".into()))?
            .map_err(|_| NoetError::Network("レスポンスの受信に失敗しました".into()))?;

        // Check for error
        if response.status == "error" {
            if let Some(err) = &response.error {
                return Err(NoetError::Extension(format!(
                    "{}: {}",
                    err.code, err.message
                )));
            }
        }

        Ok(response)
    }

    /// Ping the extension to check connection
    pub async fn ping(&self) -> Result<String> {
        let response = self.send_command("ping", None).await?;

        let version = response
            .data
            .and_then(|d| d.get("version").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(version)
    }

    /// Check authentication status
    pub async fn check_auth(&self) -> Result<AuthStatusData> {
        let response = self.send_command("check_auth", None).await?;

        let data = response
            .data
            .ok_or_else(|| NoetError::Extension("No data in response".into()))?;

        serde_json::from_value(data)
            .map_err(|e| NoetError::Extension(format!("Failed to parse auth data: {e}")))
    }

    /// List articles
    pub async fn list_articles(&self) -> Result<ArticleListData> {
        let response = self.send_command("list_articles", None).await?;

        let data = response
            .data
            .ok_or_else(|| NoetError::Extension("No data in response".into()))?;

        serde_json::from_value(data)
            .map_err(|e| NoetError::Extension(format!("Failed to parse article list: {e}")))
    }

    /// Get a single article
    pub async fn get_article(&self, username: &str, key: &str) -> Result<ArticleData> {
        let params = serde_json::json!({
            "username": username,
            "key": key
        });

        let response = self.send_command("get_article", Some(params)).await?;

        let data = response
            .data
            .ok_or_else(|| NoetError::Extension("No data in response".into()))?;

        serde_json::from_value(data)
            .map_err(|e| NoetError::Extension(format!("Failed to parse article: {e}")))
    }

    /// Create a new article
    pub async fn create_article(
        &self,
        title: &str,
        body: &str,
        tags: &[String],
        draft: bool,
    ) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "title": title,
            "body": body,
            "tags": tags,
            "draft": draft
        });

        let response = self.send_command("create_article", Some(params)).await?;

        response
            .data
            .ok_or_else(|| NoetError::Extension("No data in response".into()))
    }

    /// Update an existing article
    pub async fn update_article(
        &self,
        key: &str,
        title: &str,
        body: &str,
        tags: Option<&[String]>,
        draft: bool,
    ) -> Result<serde_json::Value> {
        let mut params = serde_json::json!({
            "key": key,
            "title": title,
            "body": body,
            "draft": draft
        });

        if let Some(t) = tags {
            params["tags"] = serde_json::json!(t);
        }

        let response = self.send_command("update_article", Some(params)).await?;

        response
            .data
            .ok_or_else(|| NoetError::Extension("No data in response".into()))
    }

    /// Delete an article
    pub async fn delete_article(&self, key: &str) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "key": key
        });

        let response = self.send_command("delete_article", Some(params)).await?;

        response
            .data
            .ok_or_else(|| NoetError::Extension("No data in response".into()))
    }

    /// Set debug mode
    #[allow(dead_code)]
    pub async fn set_debug_mode(&self, enabled: bool) -> Result<()> {
        let params = serde_json::json!({
            "enabled": enabled
        });

        self.send_command("set_debug_mode", Some(params)).await?;
        Ok(())
    }
}
