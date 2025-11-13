use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(Some(s)),
        Value::Number(n) => Ok(Some(n.to_string())),
        Value::Null => Ok(None),
        _ => Err(D::Error::custom("expected string, number, or null")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    pub name: String,

    #[serde(default)]
    pub body: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArticleStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashtag_notes: Option<Vec<Hashtag>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_count: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArticleStatus {
    Published,
    Draft,
    Scheduled,
}

impl<'de> Deserialize<'de> for ArticleStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "published" => Ok(ArticleStatus::Published),
            "draft" => Ok(ArticleStatus::Draft),
            "scheduled" => Ok(ArticleStatus::Scheduled),
            "" => Ok(ArticleStatus::Draft), // Empty string defaults to Draft
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["published", "draft", "scheduled"],
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashtag {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Magazine {
    pub key: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub urlname: String,
    pub nickname: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_profile_image_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub follower_count: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub following_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Like {
    pub id: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Stats {
    pub pv: u32,
    pub read: u32,
    pub like: u32,
    pub comment: u32,
}

// API Request/Response types
#[derive(Debug, Serialize)]
pub struct CreateArticleRequest {
    pub name: String,
    pub body: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArticleStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashtag_notes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateArticleResponse {
    pub data: Article,
}

#[derive(Debug, Serialize)]
pub struct UpdateArticleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArticleStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashtag_notes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ArticleListResponse {
    pub data: ArticleListData,
}

#[derive(Debug, Deserialize)]
pub struct ArticleListData {
    pub contents: Vec<Article>,

    #[serde(default)]
    #[allow(dead_code)]
    pub is_last_page: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SearchResponse {
    pub data: SearchData,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SearchData {
    pub contents: Vec<Article>,

    #[serde(default)]
    pub total: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_status_serialization() {
        let status = ArticleStatus::Published;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"published\"");

        let status = ArticleStatus::Draft;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"draft\"");

        let status = ArticleStatus::Scheduled;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"scheduled\"");
    }

    #[test]
    fn test_article_status_deserialization() {
        let status: ArticleStatus = serde_json::from_str("\"published\"").unwrap();
        assert_eq!(status, ArticleStatus::Published);

        let status: ArticleStatus = serde_json::from_str("\"draft\"").unwrap();
        assert_eq!(status, ArticleStatus::Draft);

        let status: ArticleStatus = serde_json::from_str("\"scheduled\"").unwrap();
        assert_eq!(status, ArticleStatus::Scheduled);
    }

    #[test]
    fn test_article_creation() {
        let article = Article {
            id: Some("123".to_string()),
            key: Some("test-key".to_string()),
            name: "Test Article".to_string(),
            body: "Test body".to_string(),
            status: Some(ArticleStatus::Draft),
            hashtag_notes: None,
            publish_at: None,
            like_count: Some(10),
            comment_count: Some(5),
            read_count: Some(100),
        };

        assert_eq!(article.id, Some("123".to_string()));
        assert_eq!(article.name, "Test Article");
        assert_eq!(article.status, Some(ArticleStatus::Draft));
        assert_eq!(article.like_count, Some(10));
    }

    #[test]
    fn test_hashtag_creation() {
        let hashtag = Hashtag {
            name: "rust".to_string(),
            note_count: Some(42),
        };

        assert_eq!(hashtag.name, "rust");
        assert_eq!(hashtag.note_count, Some(42));
    }

    #[test]
    fn test_create_article_request_serialization() {
        let request = CreateArticleRequest {
            name: "Test".to_string(),
            body: "Body".to_string(),
            status: Some(ArticleStatus::Published),
            hashtag_notes: Some(vec!["rust".to_string(), "cli".to_string()]),
            publish_at: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "Test");
        assert_eq!(json["body"], "Body");
        assert_eq!(json["status"], "published");
        assert!(json["hashtag_notes"].is_array());
    }
}
