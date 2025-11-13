use crate::api::client::NoteClient;
use crate::error::Result;
use crate::models::Magazine;
use serde_json::json;

impl NoteClient {
    /// Get magazine details
    #[allow(dead_code)]
    pub async fn get_magazine(&self, magazine_key: &str) -> Result<Magazine> {
        let path = format!("/api/v1/magazines/{}", magazine_key);
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let magazine: Magazine = serde_json::from_value(data.clone())?;
            Ok(magazine)
        } else {
            Err(crate::error::NoetError::ApiError {
                status: 404,
                message: format!("マガジン '{}' が見つかりません", magazine_key),
            })
        }
    }

    /// Add article to magazine
    pub async fn add_to_magazine(
        &self,
        magazine_key: &str,
        note_id: &str,
        note_key: &str,
    ) -> Result<()> {
        let path = format!("/api/v1/our/magazines/{}/notes", magazine_key);
        let body = json!({
            "note_id": note_id,
            "note_key": note_key,
        });

        self.post(&path, body).await?;
        Ok(())
    }

    /// Remove article from magazine
    pub async fn remove_from_magazine(&self, magazine_key: &str, note_key: &str) -> Result<()> {
        let path = format!("/api/v1/our/magazines/{}/notes/{}", magazine_key, note_key);
        self.delete(&path).await?;
        Ok(())
    }

    /// Search magazines
    #[allow(dead_code)]
    pub async fn search_magazines(&self, query: &str, page: u32) -> Result<Vec<Magazine>> {
        let path = format!(
            "/api/v3/searches?context=magazine&q={}&start={}",
            query,
            page * 10
        );
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data").and_then(|d| d.get("contents")) {
            let magazines: Vec<Magazine> = serde_json::from_value(data.clone())?;
            Ok(magazines)
        } else {
            Ok(Vec::new())
        }
    }
}
