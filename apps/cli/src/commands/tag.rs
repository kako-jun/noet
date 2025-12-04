use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;

pub async fn list_tags(page: u32) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "ハッシュタグを取得中...".cyan());

    let hashtags = client.list_hashtags(page).await?;

    if hashtags.is_empty() {
        println!("{}", "ハッシュタグが見つかりませんでした。".yellow());
        return Ok(());
    }

    println!("\n{} 件のハッシュタグが見つかりました:\n", hashtags.len());

    for tag in hashtags {
        println!("  {} {}", "#".dimmed(), tag.name.bold());
        if let Some(count) = tag.note_count {
            println!("    {} 件の記事", count.to_string().dimmed());
        }
    }

    Ok(())
}

pub async fn suggest_tags(keyword: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!(
        "{}",
        format!("'{keyword}' に一致するタグを検索中...").cyan()
    );

    let hashtags = client.search_hashtags(keyword).await?;

    if hashtags.is_empty() {
        println!(
            "{}",
            "一致するハッシュタグが見つかりませんでした。".yellow()
        );
        return Ok(());
    }

    println!("\n{} 件の一致するハッシュタグ:\n", hashtags.len());

    for tag in hashtags {
        println!("  {} {}", "#".dimmed(), tag.name.green());
        if let Some(count) = tag.note_count {
            println!("    {} 件の記事", count.to_string().dimmed());
        }
    }

    Ok(())
}
