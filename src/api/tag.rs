use crate::api::client::NoteClient;
use crate::error::Result;
use crate::models::Hashtag;

impl NoteClient {
    /// List all available hashtags
    pub async fn list_hashtags(&self, page: u32) -> Result<Vec<Hashtag>> {
        let path = format!("/api/v2/hashtags?page={}", page);
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let hashtags: Vec<Hashtag> = serde_json::from_value(data.clone())?;
            Ok(hashtags)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get hashtag details
    #[allow(dead_code)]
    pub async fn get_hashtag(&self, hashtag_name: &str) -> Result<Hashtag> {
        let path = format!("/api/v2/hashtags/{}", hashtag_name);
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data") {
            let hashtag: Hashtag = serde_json::from_value(data.clone())?;
            Ok(hashtag)
        } else {
            Err(crate::error::NoetError::ApiError {
                status: 404,
                message: format!("ハッシュタグ '{}' が見つかりません", hashtag_name),
            })
        }
    }

    /// Search hashtags by query
    pub async fn search_hashtags(&self, query: &str) -> Result<Vec<Hashtag>> {
        // Use the article search API to find hashtags
        let path = format!("/api/v3/searches?context=note&q=%23{}", query);
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        // Extract unique hashtags from articles
        let mut hashtags = Vec::new();
        if let Some(contents) = json.get("data").and_then(|d| d.get("contents")) {
            if let Some(articles) = contents.as_array() {
                for article in articles {
                    if let Some(tags) = article.get("hashtag_notes") {
                        if let Some(tag_array) = tags.as_array() {
                            for tag in tag_array {
                                if let Some(name) = tag.get("name").and_then(|n| n.as_str()) {
                                    if name.to_lowercase().contains(&query.to_lowercase()) {
                                        let hashtag = Hashtag {
                                            name: name.to_string(),
                                            note_count: tag
                                                .get("note_count")
                                                .and_then(|c| c.as_u64())
                                                .map(|c| c as u32),
                                        };
                                        hashtags.push(hashtag);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove duplicates
        hashtags.sort_by(|a, b| a.name.cmp(&b.name));
        hashtags.dedup_by(|a, b| a.name == b.name);

        Ok(hashtags)
    }
}
