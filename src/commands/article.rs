use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;
use crate::models::{Article, ArticleStatus};
use colored::Colorize;
use dialoguer::{Confirm, Input};
use std::fs;
use std::path::{Path, PathBuf};

const ARTICLE_TEMPLATE: &str = r#"---
title: {title}
status: draft
tags: []
---

# {title}

Write your article content here...
"#;

pub async fn new_article(title: Option<String>) -> Result<()> {
    let article_title = match title {
        Some(t) => t,
        None => Input::<String>::new()
            .with_prompt("Article title")
            .interact_text()?,
    };

    let filename = article_title
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let filepath = PathBuf::from(format!("{}.md", filename));

    if filepath.exists() {
        let overwrite = Confirm::new()
            .with_prompt(format!("File '{}' already exists. Overwrite?", filepath.display()))
            .interact()?;

        if !overwrite {
            println!("{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    let content = ARTICLE_TEMPLATE.replace("{title}", &article_title);
    fs::write(&filepath, content)?;

    println!("{} {}", "Created article:".green(), filepath.display());

    Ok(())
}

pub async fn publish_article(filepath: &Path, as_draft: bool) -> Result<()> {
    if !filepath.exists() {
        return Err(crate::error::NoetError::FileNotFound(filepath.display().to_string()));
    }

    let content = fs::read_to_string(filepath)?;
    let (frontmatter, body) = parse_markdown(&content)?;

    let title = frontmatter
        .get("title")
        .ok_or_else(|| crate::error::NoetError::MissingField("title".to_string()))?
        .clone();

    let status = if as_draft {
        Some(ArticleStatus::Draft)
    } else {
        frontmatter
            .get("status")
            .and_then(|s| match s.as_str() {
                "published" => Some(ArticleStatus::Published),
                "draft" => Some(ArticleStatus::Draft),
                _ => None,
            })
    };

    let tags = frontmatter.get("tags").and_then(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .into()
    });

    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "Publishing article to Note...".cyan());

    let article = client.create_article(title, body, status, tags).await?;

    println!("{} {}", "Article published:".green(), article.key.unwrap_or_default());
    if let Some(id) = article.id {
        println!("{} {}", "Article ID:".green(), id);
    }

    Ok(())
}

pub async fn edit_article(article_id: &str, filepath: &Path) -> Result<()> {
    if !filepath.exists() {
        return Err(crate::error::NoetError::FileNotFound(filepath.display().to_string()));
    }

    let content = fs::read_to_string(filepath)?;
    let (frontmatter, body) = parse_markdown(&content)?;

    let title = frontmatter.get("title").cloned();
    let status = frontmatter.get("status").and_then(|s| match s.as_str() {
        "published" => Some(ArticleStatus::Published),
        "draft" => Some(ArticleStatus::Draft),
        _ => None,
    });

    let tags = frontmatter.get("tags").and_then(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .into()
    });

    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

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
            .with_prompt(format!("Delete article '{}'? This cannot be undone.", article_id))
            .interact()?;

        if !confirm {
            println!("{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "Deleting article...".cyan());

    client.delete_article(article_id).await?;

    println!("{} {}", "Article deleted:".green(), article_id);

    Ok(())
}

pub async fn list_articles(username: &str, page: u32) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

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
                        value.trim().trim_matches(|c| c == '"' || c == '\'').to_string(),
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
