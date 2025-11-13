use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::{NoetError, Result};
use crate::models::ArticleStatus;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub async fn export_articles(
    article_key: Option<String>,
    all: bool,
    username: Option<String>,
    output: Option<PathBuf>,
    page: u32,
) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    if all {
        // Export all articles from user
        let username = username.ok_or_else(|| {
            NoetError::InvalidInput("Username is required when using --all flag".to_string())
        })?;

        let output_dir = output.unwrap_or_else(|| PathBuf::from("./exports"));

        export_all_articles(&client, &username, &output_dir, page).await?;
    } else if let Some(key) = article_key {
        // Export single article
        let output_file = output.unwrap_or_else(|| PathBuf::from(format!("{}.md", key)));

        export_single_article(&client, &key, &output_file).await?;
    } else {
        return Err(NoetError::InvalidInput(
            "Either specify an article key or use --all flag with --username".to_string(),
        ));
    }

    Ok(())
}

async fn export_single_article(
    client: &NoteClient,
    article_key: &str,
    output_file: &PathBuf,
) -> Result<()> {
    println!(
        "{}",
        format!("記事 '{}' をエクスポート中...", article_key).cyan()
    );

    let article = client.get_article(article_key).await?;

    let markdown = generate_markdown(&article);

    // Create parent directory if needed
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_file, markdown)?;

    println!(
        "{} 記事を {} にエクスポートしました",
        "✓".green(),
        output_file.display()
    );

    Ok(())
}

async fn export_all_articles(
    client: &NoteClient,
    username: &str,
    output_dir: &PathBuf,
    start_page: u32,
) -> Result<()> {
    println!(
        "{}",
        format!("ユーザー '{}' のすべての記事をエクスポート中...", username).cyan()
    );

    fs::create_dir_all(output_dir)?;

    let mut current_page = start_page;
    let mut total_exported = 0;

    loop {
        println!(
            "\n{}",
            format!("ページ {} を取得中...", current_page).dimmed()
        );

        let articles = client.list_articles(username, current_page).await?;

        if articles.is_empty() {
            break;
        }

        for article in articles {
            let _key = article
                .key
                .as_ref()
                .ok_or_else(|| NoetError::MissingField("記事キーがありません".to_string()))?;

            // Sanitize filename
            let filename = sanitize_filename(&article.name);
            let output_file = output_dir.join(format!("{}.md", filename));

            let markdown = generate_markdown(&article);
            fs::write(&output_file, markdown)?;

            println!(
                "  {} エクスポート完了: {} -> {}",
                "✓".green(),
                article.name.bold(),
                output_file.display().to_string().dimmed()
            );

            total_exported += 1;
        }

        current_page += 1;
    }

    println!(
        "\n{} {} 件の記事を {} にエクスポートしました",
        "✓".green().bold(),
        total_exported.to_string().bold(),
        output_dir.display()
    );

    Ok(())
}

fn generate_markdown(article: &crate::models::Article) -> String {
    let mut frontmatter = String::from("---\n");

    frontmatter.push_str(&format!("title: {}\n", article.name));

    if let Some(ref status) = article.status {
        let status_str = match status {
            ArticleStatus::Published => "published",
            ArticleStatus::Draft => "draft",
            ArticleStatus::Scheduled => "scheduled",
        };
        frontmatter.push_str(&format!("status: {}\n", status_str));
    }

    if let Some(ref hashtags) = article.hashtag_notes {
        if !hashtags.is_empty() {
            let tags: Vec<String> = hashtags.iter().map(|h| h.name.clone()).collect();
            frontmatter.push_str(&format!("tags: {}\n", tags.join(", ")));
        }
    }

    if let Some(ref id) = article.id {
        frontmatter.push_str(&format!("article_id: {}\n", id));
    }

    if let Some(ref key) = article.key {
        frontmatter.push_str(&format!("article_key: {}\n", key));
    }

    if let Some(ref published_at) = article.publish_at {
        frontmatter.push_str(&format!("published_at: {}\n", published_at.to_rfc3339()));
    }

    if let Some(like_count) = article.like_count {
        frontmatter.push_str(&format!("like_count: {}\n", like_count));
    }

    if let Some(comment_count) = article.comment_count {
        frontmatter.push_str(&format!("comment_count: {}\n", comment_count));
    }

    if let Some(read_count) = article.read_count {
        frontmatter.push_str(&format!("read_count: {}\n", read_count));
    }

    frontmatter.push_str("---\n\n");

    format!("{}{}", frontmatter, article.body)
}

fn sanitize_filename(name: &str) -> String {
    // Replace invalid filename characters
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello World"), "Hello World");
        assert_eq!(sanitize_filename("Test/File:Name*"), "Test-File-Name-");
        assert_eq!(
            sanitize_filename("Windows\\Path<>File"),
            "Windows-Path--File"
        );
    }

    #[test]
    fn test_generate_markdown_basic() {
        let article = crate::models::Article {
            id: Some("123".to_string()),
            key: Some("test-article".to_string()),
            name: "Test Article".to_string(),
            body: "This is the body.".to_string(),
            status: Some(ArticleStatus::Published),
            hashtag_notes: None,
            publish_at: None,
            like_count: Some(10),
            comment_count: Some(5),
            read_count: Some(100),
        };

        let markdown = generate_markdown(&article);

        assert!(markdown.contains("title: Test Article"));
        assert!(markdown.contains("status: published"));
        assert!(markdown.contains("article_id: 123"));
        assert!(markdown.contains("article_key: test-article"));
        assert!(markdown.contains("like_count: 10"));
        assert!(markdown.contains("This is the body."));
    }
}
