use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::commands::template;
use crate::config::Config;
use crate::error::Result;
use crate::models::{Article, ArticleStatus};
use crate::workspace;
use colored::Colorize;
use dialoguer::{Confirm, Input};
use std::env;
use std::fs;
use std::path::Path;

const ARTICLE_TEMPLATE: &str = r#"---
title: {title}
status: draft
tags: []
---

# {title}

Write your article content here...
"#;

/// Helper function to create a NoteClient instance
fn create_client() -> Result<NoteClient> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    NoteClient::new(config, credentials)
}

/// Helper function to parse status from frontmatter
fn parse_status(frontmatter: &std::collections::HashMap<String, String>) -> Option<ArticleStatus> {
    frontmatter.get("status").and_then(|s| match s.as_str() {
        "published" => Some(ArticleStatus::Published),
        "draft" => Some(ArticleStatus::Draft),
        _ => None,
    })
}

/// Helper function to parse tags from frontmatter
fn parse_tags(frontmatter: &std::collections::HashMap<String, String>) -> Option<Vec<String>> {
    frontmatter.get("tags").and_then(|t| {
        let tags: Vec<_> = t
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if tags.is_empty() {
            None
        } else {
            Some(tags)
        }
    })
}

/// Helper function to read and parse a markdown file
fn load_markdown_file(
    filepath: &Path,
) -> Result<(std::collections::HashMap<String, String>, String)> {
    if !filepath.exists() {
        return Err(crate::error::NoetError::FileNotFound(
            filepath.display().to_string(),
        ));
    }

    let content = fs::read_to_string(filepath)?;
    parse_markdown(&content)
}

/// Helper function to generate filename from title
fn generate_filename(title: &str) -> String {
    title
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

pub async fn new_article(title: Option<String>, template_name: Option<String>) -> Result<()> {
    // Check if in workspace
    if !workspace::is_in_workspace() {
        println!(
            "{} Not in a noet workspace. Run {} to initialize.",
            "Warning:".yellow(),
            "noet init".cyan()
        );
    }

    let article_title = match title {
        Some(t) => t,
        None => Input::<String>::new()
            .with_prompt("Article title")
            .interact_text()?,
    };

    let filename = generate_filename(&article_title);

    // Create file in current directory
    let current_dir = env::current_dir()?;
    let filepath = current_dir.join(format!("{}.md", filename));

    if filepath.exists() {
        let overwrite = Confirm::new()
            .with_prompt(format!(
                "File '{}' already exists. Overwrite?",
                filepath.display()
            ))
            .interact()?;

        if !overwrite {
            println!("{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    let content = if let Some(ref template) = template_name {
        // Use template
        println!("{}", format!("Using template '{}'...", template).cyan());
        template::load_template(template, &article_title)?
    } else {
        // Use default template
        ARTICLE_TEMPLATE.replace("{title}", &article_title)
    };

    fs::write(&filepath, content)?;

    println!("{} {}", "Created article:".green(), filepath.display());

    Ok(())
}

pub async fn publish_article(filepath: &Path, as_draft: bool, force: bool) -> Result<()> {
    let (frontmatter, body) = load_markdown_file(filepath)?;

    let title = frontmatter
        .get("title")
        .ok_or_else(|| crate::error::NoetError::MissingField("title".to_string()))?
        .clone();

    let status = if as_draft {
        Some(ArticleStatus::Draft)
    } else {
        parse_status(&frontmatter)
    };

    let tags = parse_tags(&frontmatter);

    let client = create_client()?;

    // Check if this is an update (article_key or article_id exists in frontmatter)
    let article_key = frontmatter.get("article_key");
    let is_update = article_key.is_some();

    // Show diff for updates (unless --force is used)
    if is_update && !force {
        let key = article_key.unwrap();
        println!("{}", format!("Fetching remote article '{}'...", key).cyan());

        match client.get_article(key).await {
            Ok(remote_article) => {
                // Show TUI diff
                let tui_title = format!("Publishing: {} (Article Key: {})", title, key);
                let should_publish =
                    crate::tui_diff::show_diff_tui(&tui_title, &remote_article.body, &body)?;

                if !should_publish {
                    println!("{}", "Cancelled.".yellow());
                    return Ok(());
                }
            }
            Err(e) => {
                println!(
                    "{} Could not fetch remote article: {}",
                    "Warning:".yellow(),
                    e
                );
                println!("{}", "Publishing anyway...".dimmed());
            }
        }
    }

    println!("{}", "Publishing article to Note...".cyan());

    let article = client
        .create_article(title.clone(), body, status, tags)
        .await?;

    println!(
        "{} {}",
        "Article published:".green(),
        article.key.unwrap_or_default()
    );
    if let Some(id) = article.id {
        println!("{} {}", "Article ID:".green(), id);
    }

    Ok(())
}

pub async fn show_diff(filepath: &Path) -> Result<()> {
    let (frontmatter, body) = load_markdown_file(filepath)?;

    let title = frontmatter
        .get("title")
        .ok_or_else(|| crate::error::NoetError::MissingField("title".to_string()))?;

    let article_key = frontmatter.get("article_key").ok_or_else(|| {
        crate::error::NoetError::MissingField(
            "article_key not found. This file may not have been published yet.".to_string(),
        )
    })?;

    let client = create_client()?;

    println!(
        "{}",
        format!("Fetching remote article '{}'...", article_key).cyan()
    );

    let remote_article = client.get_article(article_key).await?;

    let tui_title = format!("Diff: {} (Article Key: {})", title, article_key);
    crate::tui_diff::show_diff_tui(&tui_title, &remote_article.body, &body)?;

    Ok(())
}

pub async fn edit_article(article_id: &str, filepath: &Path) -> Result<()> {
    let (frontmatter, body) = load_markdown_file(filepath)?;

    let title = frontmatter.get("title").cloned();
    let status = parse_status(&frontmatter);
    let tags = parse_tags(&frontmatter);

    let client = create_client()?;

    println!("{}", "Updating article on Note...".cyan());

    client
        .update_article(article_id, title, Some(body), status, tags)
        .await?;

    println!("{} {}", "Article updated:".green(), article_id);

    Ok(())
}

pub async fn delete_article(article_id: &str, force: bool) -> Result<()> {
    if !force {
        let confirm = Confirm::new()
            .with_prompt(format!(
                "Delete article '{}'? This cannot be undone.",
                article_id
            ))
            .interact()?;

        if !confirm {
            println!("{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    let client = create_client()?;

    println!("{}", "Deleting article...".cyan());

    client.delete_article(article_id).await?;

    println!("{} {}", "Article deleted:".green(), article_id);

    Ok(())
}

pub async fn list_articles(username: &str, page: u32) -> Result<()> {
    let client = create_client()?;

    println!("{}", "Fetching articles...".cyan());

    let articles = client.list_articles(username, page).await?;

    if articles.is_empty() {
        println!("{}", "No articles found.".yellow());
        return Ok(());
    }

    println!("\n{} articles found:\n", articles.len());

    for article in articles {
        print_article_summary(&article);
        println!();
    }

    Ok(())
}

fn parse_markdown(content: &str) -> Result<(std::collections::HashMap<String, String>, String)> {
    let mut frontmatter = std::collections::HashMap::new();
    let body;

    if content.starts_with("---") {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 3 {
            // Parse frontmatter
            for line in parts[1].lines() {
                if let Some((key, value)) = line.split_once(':') {
                    frontmatter.insert(
                        key.trim().to_string(),
                        value
                            .trim()
                            .trim_matches(|c| c == '"' || c == '\'')
                            .to_string(),
                    );
                }
            }
            body = parts[2].trim().to_string();
        } else {
            body = content.to_string();
        }
    } else {
        body = content.to_string();
    }

    Ok((frontmatter, body))
}

fn print_article_summary(article: &Article) {
    println!("{}", article.name.bold());

    if let Some(ref key) = article.key {
        println!("  {} {}", "Key:".dimmed(), key);
    }

    if let Some(ref id) = article.id {
        println!("  {} {}", "ID:".dimmed(), id);
    }

    if let Some(ref status) = article.status {
        let status_str = match status {
            ArticleStatus::Published => "Published".green(),
            ArticleStatus::Draft => "Draft".yellow(),
            ArticleStatus::Scheduled => "Scheduled".cyan(),
        };
        println!("  {} {}", "Status:".dimmed(), status_str);
    }

    if let Some(like_count) = article.like_count {
        println!("  {} {}", "Likes:".dimmed(), like_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_with_frontmatter() {
        let content = r#"---
title: Test Article
status: published
tags: rust, cli
---

# Test Article

This is the body."#;

        let (frontmatter, body) = parse_markdown(content).unwrap();

        assert_eq!(frontmatter.get("title"), Some(&"Test Article".to_string()));
        assert_eq!(frontmatter.get("status"), Some(&"published".to_string()));
        assert_eq!(frontmatter.get("tags"), Some(&"rust, cli".to_string()));
        assert!(body.contains("# Test Article"));
        assert!(body.contains("This is the body."));
    }

    #[test]
    fn test_parse_markdown_without_frontmatter() {
        let content = "# Just Content\n\nNo frontmatter here.";

        let (frontmatter, body) = parse_markdown(content).unwrap();

        assert!(frontmatter.is_empty());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_markdown_with_quotes() {
        let content = r#"---
title: "Article with Quotes"
status: 'draft'
---

Body content"#;

        let (frontmatter, body) = parse_markdown(content).unwrap();

        assert_eq!(
            frontmatter.get("title"),
            Some(&"Article with Quotes".to_string())
        );
        assert_eq!(frontmatter.get("status"), Some(&"draft".to_string()));
        assert_eq!(body, "Body content");
    }

    #[test]
    fn test_parse_markdown_empty_body() {
        let content = r#"---
title: Empty Body
---
"#;

        let (frontmatter, body) = parse_markdown(content).unwrap();

        assert_eq!(frontmatter.get("title"), Some(&"Empty Body".to_string()));
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_markdown_malformed_frontmatter() {
        let content = "---\ninvalid frontmatter\n---\n\nBody";

        let (frontmatter, body) = parse_markdown(content).unwrap();

        // Should handle malformed frontmatter gracefully
        assert!(frontmatter.is_empty());
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_parse_status_published() {
        let mut map = std::collections::HashMap::new();
        map.insert("status".to_string(), "published".to_string());
        assert_eq!(parse_status(&map), Some(ArticleStatus::Published));
    }

    #[test]
    fn test_parse_status_draft() {
        let mut map = std::collections::HashMap::new();
        map.insert("status".to_string(), "draft".to_string());
        assert_eq!(parse_status(&map), Some(ArticleStatus::Draft));
    }

    #[test]
    fn test_parse_status_invalid() {
        let mut map = std::collections::HashMap::new();
        map.insert("status".to_string(), "invalid".to_string());
        assert_eq!(parse_status(&map), None);
    }

    #[test]
    fn test_parse_status_missing() {
        let map = std::collections::HashMap::new();
        assert_eq!(parse_status(&map), None);
    }

    #[test]
    fn test_parse_tags_single() {
        let mut map = std::collections::HashMap::new();
        map.insert("tags".to_string(), "rust".to_string());
        assert_eq!(parse_tags(&map), Some(vec!["rust".to_string()]));
    }

    #[test]
    fn test_parse_tags_multiple() {
        let mut map = std::collections::HashMap::new();
        map.insert("tags".to_string(), "rust, cli, note".to_string());
        assert_eq!(
            parse_tags(&map),
            Some(vec![
                "rust".to_string(),
                "cli".to_string(),
                "note".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_tags_with_spaces() {
        let mut map = std::collections::HashMap::new();
        map.insert("tags".to_string(), "  rust  ,  cli  ,  note  ".to_string());
        assert_eq!(
            parse_tags(&map),
            Some(vec![
                "rust".to_string(),
                "cli".to_string(),
                "note".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_tags_empty() {
        let mut map = std::collections::HashMap::new();
        map.insert("tags".to_string(), "".to_string());
        assert_eq!(parse_tags(&map), None);
    }

    #[test]
    fn test_parse_tags_missing() {
        let map = std::collections::HashMap::new();
        assert_eq!(parse_tags(&map), None);
    }

    #[test]
    fn test_generate_filename_simple() {
        assert_eq!(generate_filename("My Article"), "my-article");
    }

    #[test]
    fn test_generate_filename_with_special_chars() {
        assert_eq!(
            generate_filename("Hello! World? (Test)"),
            "hello-world-test"
        );
    }

    #[test]
    fn test_generate_filename_with_numbers() {
        assert_eq!(generate_filename("Article 123"), "article-123");
    }

    #[test]
    fn test_generate_filename_multiple_spaces() {
        assert_eq!(generate_filename("Multiple   Spaces"), "multiple---spaces");
    }

    #[test]
    fn test_generate_filename_japanese() {
        // Japanese characters are kept as-is (unicode alphanumeric)
        assert_eq!(generate_filename("日本語テスト"), "日本語テスト");
    }

    #[test]
    fn test_generate_filename_mixed() {
        // Mix of alphanumeric and special chars
        assert_eq!(
            generate_filename("Test 123 & Article!"),
            "test-123--article"
        );
    }
}
