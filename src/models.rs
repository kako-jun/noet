use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    pub name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArticleStatus {
    Published,
    Draft,
    Scheduled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashtag {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct Like {
    pub id: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub is_last_page: bool,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub data: SearchData,
}

#[derive(Debug, Deserialize)]
pub struct SearchData {
    pub contents: Vec<Article>,

    #[serde(default)]
    pub total: u32,
}
