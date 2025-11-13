use crate::api::client::NoteClient;
use crate::error::Result;
use crate::models::{
    Article, ArticleListResponse, ArticleStatus, CreateArticleRequest, UpdateArticleRequest,
};
use serde_json::json;

/// Helper function to extract article from JSON response
fn extract_article_from_response(json: serde_json::Value) -> Result<Article> {
    if let Some(data) = json.get("data") {
        let article: Article = serde_json::from_value(data.clone()).map_err(|e| {
            eprintln!("JSONパースエラー: {}", e);
            eprintln!(
                "レスポンス: {}",
                serde_json::to_string_pretty(&json).unwrap_or_default()
            );
            e
        })?;
        Ok(article)
    } else {
        eprintln!("レスポンスに'data'フィールドがありません");
        eprintln!(
            "レスポンス全体: {}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
        Err(crate::error::NoetError::ApiError {
            status: 500,
            message: "Unexpected response format".to_string(),
        })
    }
}

impl NoteClient {
    /// Create a new article (draft or published)
    pub async fn create_article(
        &self,
        name: String,
        body: String,
        status: Option<ArticleStatus>,
        hashtags: Option<Vec<String>>,
    ) -> Result<Article> {
        let request = CreateArticleRequest {
            name: name.clone(),
            body: body.clone(),
            status,
            hashtag_notes: hashtags,
            publish_at: None,
        };

        let response = self.post("/api/v1/text_notes", request).await?;
        let json: serde_json::Value = response.json().await?;

        // For create response, extract minimal info and return a partial Article
        if let Some(data) = json.get("data") {
            let id = data
                .get("id")
                .and_then(|v| v.as_i64())
                .map(|n| n.to_string());
            let key = data.get("key").and_then(|v| v.as_str()).map(String::from);

            Ok(Article {
                id,
                key,
                name,
                body,
                status: None,
                hashtag_notes: None,
                publish_at: None,
                like_count: None,
                comment_count: None,
                read_count: None,
            })
        } else {
            extract_article_from_response(json)
        }
    }

    /// Save article as draft
    #[allow(dead_code)]
    pub async fn save_draft(
        &self,
        id: Option<String>,
        name: String,
        body: String,
    ) -> Result<Article> {
        let mut path = "/api/v1/text_notes/draft_save".to_string();
        if let Some(article_id) = id {
            path = format!("{}?id={}", path, article_id);
        }

        let request = json!({
            "name": name,
            "body": body,
        });

        let response = self.post(&path, request).await?;
        let json: serde_json::Value = response.json().await?;
        extract_article_from_response(json)
    }

    /// Update an existing article
    pub async fn update_article(
        &self,
        article_id: &str,
        name: Option<String>,
        body: Option<String>,
        status: Option<ArticleStatus>,
        hashtags: Option<Vec<String>>,
    ) -> Result<Article> {
        let request = UpdateArticleRequest {
            name,
            body,
            status,
            hashtag_notes: hashtags,
        };

        let path = format!("/api/v1/text_notes/{}", article_id);
        let response = self.put(&path, request).await?;
        let json: serde_json::Value = response.json().await?;
        extract_article_from_response(json)
    }

    /// Delete an article
    pub async fn delete_article(&self, article_id: &str) -> Result<()> {
        let path = format!("/api/v1/text_notes/{}", article_id);
        self.delete(&path).await?;
        Ok(())
    }

    /// Get article details
    pub async fn get_article(&self, article_key: &str) -> Result<Article> {
        let path = format!("/api/v3/notes/{}", article_key);
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        let mut article = extract_article_from_response(json)
            .map_err(|_| crate::error::NoetError::ArticleNotFound(article_key.to_string()))?;

        // Convert HTML body to Markdown
        if !article.body.is_empty() {
            article.body = crate::converters::convert_html_to_markdown(&article.body)?;
        }

        Ok(article)
    }

    /// List articles for a user
    pub async fn list_articles(&self, username: &str, page: u32) -> Result<Vec<Article>> {
        let path = format!(
            "/api/v2/creators/{}/contents?kind=note&page={}",
            username, page
        );
        let response = self.get(&path).await?;
        let list_response: ArticleListResponse = response.json().await?;

        Ok(list_response.data.contents)
    }

    /// Search articles by keyword
    #[allow(dead_code)]
    pub async fn search_articles(&self, query: &str, page: u32) -> Result<Vec<Article>> {
        let path = format!(
            "/api/v3/searches?context=note&q={}&start={}",
            query,
            page * 10
        );
        let response = self.get(&path).await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(data) = json.get("data").and_then(|d| d.get("contents")) {
            let articles: Vec<Article> = serde_json::from_value(data.clone())?;
            Ok(articles)
        } else {
            Ok(Vec::new())
        }
    }
}
