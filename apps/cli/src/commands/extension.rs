//! Extension-based commands for Note.com operations via browser extension

use crate::error::Result;
use crate::extension_client::ExtensionClient;
use crate::image_handler::{self, ImageData};
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
pub async fn get_article(username: &str, key: &str, save_path: Option<&Path>) -> Result<()> {
    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;

    println!("{}", "記事を取得中...".cyan());
    let article = client.get_article(username, key).await?;

    let html = article
        .html
        .as_ref()
        .ok_or_else(|| crate::error::NoetError::Extension("Article HTML not found".to_string()))?;

    // Convert HTML to markdown
    let markdown = html2md::parse_html(html);

    // If save path is specified, download images and save to file
    if let Some(save_file) = save_path {
        println!("{}", "画像をダウンロード中...".cyan());

        let (markdown_with_local_paths, header_image) =
            download_images_and_replace_urls(&markdown, save_file).await?;

        // Create frontmatter
        let mut frontmatter = format!("---\ntitle: \"{}\"\n", article.title);

        if let Some(tags) = &article.tags {
            if !tags.is_empty() {
                frontmatter.push_str(&format!(
                    "tags: [{}]\n",
                    tags.iter()
                        .map(|t| format!("\"{t}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        if let Some(header_img) = header_image {
            frontmatter.push_str(&format!("header_image: {header_img}\n"));
        }

        frontmatter.push_str(&format!("note_key: {key}\n"));
        frontmatter.push_str("---\n\n");

        let full_content = format!("{frontmatter}{markdown_with_local_paths}");

        // Save to file
        fs::write(save_file, full_content)?;

        println!(
            "{} ファイルに保存しました: {}",
            "✓".green(),
            save_file.display()
        );
    } else {
        // Just display to console
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
        println!("{markdown}");
    }

    Ok(())
}

/// Download images from Note.com URLs and replace with local paths
async fn download_images_and_replace_urls(
    markdown: &str,
    save_file: &Path,
) -> Result<(String, Option<String>)> {
    use regex::Regex;

    let base_dir = save_file.parent().unwrap_or_else(|| Path::new("."));
    let images_dir = base_dir.join("images");

    // Create images directory if it doesn't exist
    if !images_dir.exists() {
        fs::create_dir_all(&images_dir)?;
    }

    let mut modified_markdown = markdown.to_string();
    let mut header_image: Option<String> = None;

    // Pattern for ![caption](url)
    let img_re = Regex::new(r"!\[([^\]]*)\]\((https://[^)]+)\)").unwrap();

    for cap in img_re.captures_iter(markdown) {
        let caption = &cap[1];
        let url = &cap[2];

        // Download image
        if let Ok(local_path) = download_image(url, &images_dir).await {
            let relative_path = format!(
                "./images/{}",
                local_path.file_name().unwrap().to_str().unwrap()
            );

            // Replace in markdown
            let old_pattern = format!("![{caption}]({url})");
            let new_pattern = format!("![{caption}]({relative_path})");
            modified_markdown = modified_markdown.replace(&old_pattern, &new_pattern);

            println!("  {} → {}", url.dimmed(), relative_path);
        }
    }

    // Check for eyecatch/header image in HTML comments or metadata
    // Note: This is a simple heuristic, might need adjustment
    let eyecatch_re = Regex::new(r#"eyecatch[^"]*"([^"]+)""#).unwrap();
    if let Some(cap) = eyecatch_re.captures(markdown) {
        let url = &cap[1];
        if let Ok(local_path) = download_image(url, &images_dir).await {
            let relative_path = format!(
                "./images/{}",
                local_path.file_name().unwrap().to_str().unwrap()
            );
            header_image = Some(relative_path);
        }
    }

    Ok((modified_markdown, header_image))
}

/// Download a single image from URL
async fn download_image(url: &str, images_dir: &Path) -> Result<std::path::PathBuf> {
    use reqwest;

    // Extract filename from URL
    let filename = url.split('/').next_back().unwrap_or("image");
    let file_path = images_dir.join(filename);

    // Download image
    let response = reqwest::get(url)
        .await
        .map_err(|e| crate::error::NoetError::Network(format!("Failed to download image: {e}")))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| crate::error::NoetError::Network(format!("Failed to read image data: {e}")))?;

    // Save to file
    fs::write(&file_path, bytes)?;

    Ok(file_path)
}

/// Create article via extension
pub async fn create_article(file: &Path, draft: bool) -> Result<()> {
    // Read the markdown file
    let content = fs::read_to_string(file)?;

    // Parse frontmatter and body
    let (title, body, tags, header_image_path) = parse_markdown_file(&content);

    // Process images from markdown
    let images = image_handler::process_images(file, &body)?;

    // Process header image if specified
    let header_image = if let Some(path_str) = header_image_path {
        let header_path = if Path::new(&path_str).is_absolute() {
            std::path::PathBuf::from(&path_str)
        } else {
            file.parent()
                .ok_or_else(|| {
                    crate::error::NoetError::InvalidInput(
                        "Cannot determine base directory".to_string(),
                    )
                })?
                .join(&path_str)
        };

        if header_path.exists() {
            let (mime_type, base64_data) = image_handler::read_image_as_base64(&header_path)?;
            let filename = header_path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    crate::error::NoetError::InvalidInput(
                        "Invalid header image filename".to_string(),
                    )
                })?
                .to_string();

            Some(ImageData {
                local_path: path_str.clone(),
                filename,
                caption: String::new(),
                mime_type,
                data: base64_data,
            })
        } else {
            eprintln!("Warning: Header image not found: {}", header_path.display());
            None
        }
    } else {
        None
    };

    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;

    let mode = if draft { "下書き" } else { "公開" };

    if !images.is_empty() || header_image.is_some() {
        let img_count = images.len();
        let has_header = if header_image.is_some() {
            " (見出し画像あり)"
        } else {
            ""
        };
        println!(
            "{}",
            format!("記事を{mode}として投稿中... (画像: {img_count}枚{has_header})").cyan()
        );
    } else {
        println!("{}", format!("記事を{mode}として投稿中...").cyan());
    }

    let result = if !images.is_empty() || header_image.is_some() {
        client
            .create_article_with_images(&title, &body, &tags, draft, &images, header_image.as_ref())
            .await?
    } else {
        client.create_article(&title, &body, &tags, draft).await?
    };

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

        // Show uploaded images
        if let Some(uploaded_images) = result.get("uploaded_images").and_then(|v| v.as_array()) {
            if !uploaded_images.is_empty() {
                println!("  アップロードされた画像:");
                for img in uploaded_images {
                    if let (Some(local_path), Some(note_url)) = (
                        img.get("local_path").and_then(|v| v.as_str()),
                        img.get("note_url").and_then(|v| v.as_str()),
                    ) {
                        println!("    {} → {}", local_path, note_url.dimmed());
                    }
                }
            }
        }

        if let Some(header_url) = result.get("header_image_url").and_then(|v| v.as_str()) {
            println!("  見出し画像: {}", header_url.dimmed());
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
    let (title, body, tags, header_image_path) = parse_markdown_file(&content);

    // Process images from markdown
    let images = image_handler::process_images(file, &body)?;

    // Process header image if specified
    let header_image = if let Some(path_str) = header_image_path {
        let header_path = if Path::new(&path_str).is_absolute() {
            std::path::PathBuf::from(&path_str)
        } else {
            file.parent()
                .ok_or_else(|| {
                    crate::error::NoetError::InvalidInput(
                        "Cannot determine base directory".to_string(),
                    )
                })?
                .join(&path_str)
        };

        if header_path.exists() {
            let (mime_type, base64_data) = image_handler::read_image_as_base64(&header_path)?;
            let filename = header_path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    crate::error::NoetError::InvalidInput(
                        "Invalid header image filename".to_string(),
                    )
                })?
                .to_string();

            Some(ImageData {
                local_path: path_str.clone(),
                filename,
                caption: String::new(),
                mime_type,
                data: base64_data,
            })
        } else {
            eprintln!("Warning: Header image not found: {}", header_path.display());
            None
        }
    } else {
        None
    };

    println!("{}", "拡張機能に接続中...".cyan());

    let client = ExtensionClient::connect().await?;

    let mode = if draft { "下書き保存" } else { "更新" };

    if !images.is_empty() || header_image.is_some() {
        let img_count = images.len();
        let has_header = if header_image.is_some() {
            " (見出し画像あり)"
        } else {
            ""
        };
        println!(
            "{}",
            format!("記事を{mode}中... (画像: {img_count}枚{has_header})").cyan()
        );
    } else {
        println!("{}", format!("記事を{mode}中...").cyan());
    }

    let result = if !images.is_empty() || header_image.is_some() {
        client
            .update_article_with_images(
                key,
                &title,
                &body,
                Some(&tags),
                draft,
                &images,
                header_image.as_ref(),
            )
            .await?
    } else {
        client
            .update_article(key, &title, &body, Some(&tags), draft)
            .await?
    };

    if result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        println!("{} 記事を{}しました", "✓".green(), mode);

        if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
            println!("  ステータス: {status}");
        }

        // Show uploaded images
        if let Some(uploaded_images) = result.get("uploaded_images").and_then(|v| v.as_array()) {
            if !uploaded_images.is_empty() {
                println!("  アップロードされた画像:");
                for img in uploaded_images {
                    if let (Some(local_path), Some(note_url)) = (
                        img.get("local_path").and_then(|v| v.as_str()),
                        img.get("note_url").and_then(|v| v.as_str()),
                    ) {
                        println!("    {} → {}", local_path, note_url.dimmed());
                    }
                }
            }
        }

        if let Some(header_url) = result.get("header_image_url").and_then(|v| v.as_str()) {
            println!("  見出し画像: {}", header_url.dimmed());
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
/// Returns (title, body, tags, header_image)
fn parse_markdown_file(content: &str) -> (String, String, Vec<String>, Option<String>) {
    let mut title = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut header_image: Option<String> = None;
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
                } else if let Some(value) = line.strip_prefix("header_image:") {
                    // header_image: path/to/image.jpg
                    let value = value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    if !value.is_empty() {
                        header_image = Some(value);
                    }
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

    (title, body, tags, header_image)
}
