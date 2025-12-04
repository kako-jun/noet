use crate::api::client::NoteClient;
use crate::error::Result;
use crate::models::User;

impl NoteClient {
    /// Get user details
    pub async fn get_user(&self, username: &str) -> Result<User> {
        let path = format!("/api/v2/creators/{username}");
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let user: User = serde_json::from_value(data.clone())?;
            Ok(user)
        } else {
            Err(crate::error::NoetError::ApiError {
                status: 404,
                message: format!("ユーザー '{username}' が見つかりません"),
            })
        }
    }

    /// Get user's followings
    #[allow(dead_code)]
    pub async fn get_followings(&self, username: &str) -> Result<Vec<User>> {
        let path = format!("/api/v1/followings/{username}/list");
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let users: Vec<User> = serde_json::from_value(data.clone())?;
            Ok(users)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get user's followers
    #[allow(dead_code)]
    pub async fn get_followers(&self, username: &str) -> Result<Vec<User>> {
        let path = format!("/api/v1/followers/{username}/list");
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let users: Vec<User> = serde_json::from_value(data.clone())?;
            Ok(users)
        } else {
            Ok(Vec::new())
        }
    }

    /// Follow a user
    #[allow(dead_code)]
    pub async fn follow_user(&self, user_key: &str) -> Result<()> {
        let path = format!("/api/v3/users/{user_key}/following");
        let body = serde_json::json!({});
        self.post(&path, body).await?;
        Ok(())
    }

    /// Search users
    #[allow(dead_code)]
    pub async fn search_users(&self, query: &str, page: u32) -> Result<Vec<User>> {
        let path = format!(
            "/api/v3/searches?context=user&q={}&start={}",
            query,
            page * 10
        );
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data").and_then(|d| d.get("contents")) {
            let users: Vec<User> = serde_json::from_value(data.clone())?;
            Ok(users)
        } else {
            Ok(Vec::new())
        }
    }
}
