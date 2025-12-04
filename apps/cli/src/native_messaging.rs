//! Native Messaging host for browser extension communication
//!
//! Chrome Native Messaging protocol:
//! - Messages are length-prefixed (4 bytes, little-endian)
//! - Message body is JSON
//! - Communication via stdin/stdout

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

/// Request from browser extension
#[derive(Debug, Deserialize)]
pub struct NativeRequest {
    pub id: String,
    pub command: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Response to browser extension
#[derive(Debug, Serialize)]
pub struct NativeResponse {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<NativeError>,
}

#[derive(Debug, Serialize)]
pub struct NativeError {
    pub code: String,
    pub message: String,
}

impl NativeResponse {
    pub fn success(id: String, data: serde_json::Value) -> Self {
        Self {
            id,
            status: "success".to_string(),
            data: Some(data),
            error: None,
        }
    }

    pub fn error(id: String, code: &str, message: &str) -> Self {
        Self {
            id,
            status: "error".to_string(),
            data: None,
            error: Some(NativeError {
                code: code.to_string(),
                message: message.to_string(),
            }),
        }
    }
}

/// Read a message from stdin using Native Messaging protocol
fn read_message() -> Result<Option<NativeRequest>> {
    let mut stdin = io::stdin().lock();

    // Read 4-byte length prefix
    let mut len_bytes = [0u8; 4];
    match stdin.read_exact(&mut len_bytes) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }

    let len = u32::from_le_bytes(len_bytes) as usize;

    // Read message body
    let mut buffer = vec![0u8; len];
    stdin.read_exact(&mut buffer)?;

    let request: NativeRequest = serde_json::from_slice(&buffer)?;
    Ok(Some(request))
}

/// Write a message to stdout using Native Messaging protocol
fn write_message(response: &NativeResponse) -> Result<()> {
    let json = serde_json::to_vec(response)?;
    let len = json.len() as u32;

    let mut stdout = io::stdout().lock();
    stdout.write_all(&len.to_le_bytes())?;
    stdout.write_all(&json)?;
    stdout.flush()?;

    Ok(())
}

/// Handle a single command from the extension
async fn handle_command(request: NativeRequest) -> NativeResponse {
    let id = request.id.clone();

    match request.command.as_str() {
        "ping" => {
            let version = env!("CARGO_PKG_VERSION");
            NativeResponse::success(
                id,
                serde_json::json!({
                    "version": version,
                    "host": "noet"
                }),
            )
        }

        "check_auth" => {
            // TODO: Implement actual auth check
            // For now, return placeholder
            NativeResponse::success(
                id,
                serde_json::json!({
                    "logged_in": false,
                    "username": null
                }),
            )
        }

        "list_articles" => {
            // TODO: This will be handled by the extension, not the host
            // The host just relays file operations
            NativeResponse::error(
                id,
                "NOT_IMPLEMENTED",
                "list_articles should be called from extension",
            )
        }

        "get_article" => NativeResponse::error(
            id,
            "NOT_IMPLEMENTED",
            "get_article should be called from extension",
        ),

        "create_article" => NativeResponse::error(
            id,
            "NOT_IMPLEMENTED",
            "create_article should be called from extension",
        ),

        "update_article" => NativeResponse::error(
            id,
            "NOT_IMPLEMENTED",
            "update_article should be called from extension",
        ),

        "delete_article" => NativeResponse::error(
            id,
            "NOT_IMPLEMENTED",
            "delete_article should be called from extension",
        ),

        "set_debug_mode" => {
            // Store debug mode state
            let enabled = request
                .params
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            // TODO: Store this somewhere
            NativeResponse::success(
                id,
                serde_json::json!({
                    "success": true,
                    "debug_mode": enabled
                }),
            )
        }

        "get_debug_mode" => {
            // TODO: Retrieve stored debug mode
            NativeResponse::success(
                id,
                serde_json::json!({
                    "debug_mode": false
                }),
            )
        }

        _ => NativeResponse::error(
            id,
            "UNKNOWN_COMMAND",
            &format!("Unknown command: {}", request.command),
        ),
    }
}

/// Main loop for Native Messaging host mode
pub async fn run() -> Result<()> {
    log::info!("Starting Native Messaging host");

    loop {
        match read_message() {
            Ok(Some(request)) => {
                log::debug!("Received request: {request:?}");
                let response = handle_command(request).await;
                write_message(&response)?;
            }
            Ok(None) => {
                // EOF - extension disconnected
                log::info!("Extension disconnected");
                break;
            }
            Err(e) => {
                log::error!("Error reading message: {e}");
                break;
            }
        }
    }

    Ok(())
}
