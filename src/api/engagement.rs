use crate::api::client::NoteClient;
use crate::error::Result;
use crate::models::{Comment, Like};
use serde_json::json;

impl NoteClient {
    /// Like an article
    pub async fn like_article(&self, note_key: &str) -> Result<()> {
        let path = format!("/api/v3/notes/{}/likes", note_key);
        self.post(&path, json!({})).await?;
        Ok(())
    }

    /// Unlike an article
    pub async fn unlike_article(&self, note_key: &str) -> Result<()> {
        let path = format!("/api/v3/notes/{}/likes", note_key);
        self.delete(&path).await?;
        Ok(())
    }

    /// Get likes for an article
    #[allow(dead_code)]
    pub async fn get_likes(&self, note_key: &str) -> Result<Vec<Like>> {
        let path = format!("/api/v3/notes/{}/likes", note_key);
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let likes: Vec<Like> = serde_json::from_value(data.clone())?;
            Ok(likes)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get comments for an article
    pub async fn get_comments(&self, note_id: &str) -> Result<Vec<Comment>> {
        let path = format!("/api/v1/note/{}/comments", note_id);
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let comments: Vec<Comment> = serde_json::from_value(data.clone())?;
            Ok(comments)
        } else {
            Ok(Vec::new())
        }
    }

    /// Post a comment on an article
    #[allow(dead_code)]
    pub async fn post_comment(&self, note_id: &str, comment: &str) -> Result<Comment> {
        let path = format!("/api/v1/note/{}/comments", note_id);
        let body = json!({
            "comment": comment,
            "acknowledgement": true,
        });

        let response = self.post(&path, body).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let comment: Comment = serde_json::from_value(data.clone())?;
            Ok(comment)
        } else {
            Err(crate::error::NoetError::ApiError {
                status: 500,
                message: "Failed to post comment".to_string(),
            })
        }
    }
}
