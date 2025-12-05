//! Extension-based commands for Note.com operations via browser extension

use crate::error::Result;
use crate::extension_client::ExtensionClient;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Check connection to browser extension
pub async fn ping() -> Result<()> {
    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;
    let version = client.ping().await?;

    println!("{} 拡張機能と接続しました", "✓".green());
    println!("  バージョン: {}", version.cyan());

    Ok(())
}

/// Check authentication status via extension
pub async fn check_auth() -> Result<()> {
    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;
    let auth = client.check_auth().await?;

    if auth.logged_in {
        println!("{} Note.com にログイン済み", "✓".green());
        if let Some(username) = auth.username {
            println!("  ユーザー名: {}", username.cyan());
        }
    } else {
        println!("{} Note.com にログインしていません", "✗".red());
        println!("  ブラウザで https://note.com/login にアクセスしてログインしてください");
    }

    Ok(())
}

/// List articles via extension
pub async fn list_articles() -> Result<()> {
    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;

    println!("{}", "記事一覧を取得中...".cyan());
    let result = client.list_articles().await?;

    println!();
    println!(
        "{} 件の記事が見つかりました",
        result.count.to_string().cyan()
    );
    println!();

    for article in result.articles {
        let status_badge = match article.status.as_deref() {
            Some("published") => "公開中".green(),
            Some("draft") => "下書き".yellow(),
            _ => "不明".dimmed(),
        };

        let key = article.key.unwrap_or_else(|| "-".to_string());
        let title = if article.title.is_empty() {
            "(無題)".dimmed().to_string()
        } else {
            article.title
        };

        println!("  [{}] {} {}", status_badge, key.cyan(), title);

        if let Some(date) = article.date {
            println!("      {}", date.dimmed());
        }
    }

    Ok(())
}

/// Get article content via extension
pub async fn get_article(username: &str, key: &str) -> Result<()> {
    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;

    println!("{}", "記事を取得中...".cyan());
    let article = client.get_article(username, key).await?;

    println!();
    println!("{} {}", "タイトル:".cyan(), article.title);

    if let Some(tags) = &article.tags {
        if !tags.is_empty() {
            println!(
                "{} {}",
                "タグ:".cyan(),
                tags.iter()
                    .map(|t| format!("#{t}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }

    if let Some(published_at) = &article.published_at {
        println!("{} {}", "公開日:".cyan(), published_at);
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());

    if let Some(html) = &article.html {
        // Convert HTML to markdown for display
        let markdown = html2md::parse_html(html);
        println!("{markdown}");
    }

    Ok(())
}

/// Create article via extension
pub async fn create_article(file: &Path, draft: bool) -> Result<()> {
    // Read the markdown file
    let content = fs::read_to_string(file)?;

    // Parse frontmatter and body
    let (title, body, tags) = parse_markdown_file(&content);

    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;

    let mode = if draft { "下書き" } else { "公開" };
    println!("{}", format!("記事を{mode}として投稿中...").cyan());

    let result = client.create_article(&title, &body, &tags, draft).await?;

    if result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        println!("{} 記事を{}しました", "✓".green(), mode);

        if let Some(url) = result.get("url").and_then(|v| v.as_str()) {
            println!("  URL: {}", url.cyan());
        }

        if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
            println!("  ステータス: {status}");
        }
    } else {
        let error = result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("不明なエラー");
        println!("{} 投稿に失敗しました: {error}", "✗".red());
    }

    Ok(())
}

/// Update article via extension
pub async fn update_article(key: &str, file: &Path, draft: bool) -> Result<()> {
    // Read the markdown file
    let content = fs::read_to_string(file)?;

    // Parse frontmatter and body
    let (title, body, tags) = parse_markdown_file(&content);

    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;

    let mode = if draft { "下書き保存" } else { "更新" };
    println!("{}", format!("記事を{mode}中...").cyan());

    let result = client
        .update_article(key, &title, &body, Some(&tags), draft)
        .await?;

    if result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        println!("{} 記事を{}しました", "✓".green(), mode);

        if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
            println!("  ステータス: {status}");
        }
    } else {
        let error = result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("不明なエラー");
        println!("{} 更新に失敗しました: {error}", "✗".red());
    }

    Ok(())
}

/// Delete article via extension
pub async fn delete_article(key: &str) -> Result<()> {
    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;

    println!("{}", "記事を削除中...".cyan());
    let result = client.delete_article(key).await?;

    if result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        println!("{} 記事を削除しました", "✓".green());
    } else {
        let error = result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("不明なエラー");
        println!("{} 削除に失敗しました: {}", "✗".red(), error);
    }

    Ok(())
}

/// Parse a markdown file with frontmatter
/// Returns (title, body, tags)
fn parse_markdown_file(content: &str) -> (String, String, Vec<String>) {
    let mut title = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut body = content.to_string();

    // Check for YAML frontmatter
    if let Some(rest) = content.strip_prefix("---") {
        if let Some(end) = rest.find("---") {
            let frontmatter = &rest[..end];
            body = rest[end + 3..].trim().to_string();

            // Parse frontmatter
            for line in frontmatter.lines() {
                let line = line.trim();
                if let Some(value) = line.strip_prefix("title:") {
                    title = value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                } else if let Some(value) = line.strip_prefix("tags:") {
                    // tags: [tag1, tag2] or tags: tag1, tag2
                    let value = value.trim().trim_matches('[').trim_matches(']');
                    tags = value
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
    }

    // If no title in frontmatter, try to extract from first H1
    if title.is_empty() {
        for line in body.lines() {
            let line = line.trim();
            if let Some(h1) = line.strip_prefix("# ") {
                title = h1.to_string();
                // Remove the H1 from body
                body = body.replacen(line, "", 1).trim().to_string();
                break;
            }
        }
    }

    (title, body, tags)
}
