use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;

pub async fn list_tags(page: u32) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "Fetching hashtags...".cyan());

    let hashtags = client.list_hashtags(page).await?;

    if hashtags.is_empty() {
        println!("{}", "No hashtags found.".yellow());
        return Ok(());
    }

    println!("\n{} hashtags found:\n", hashtags.len());

    for tag in hashtags {
        println!("  {} {}", "#".dimmed(), tag.name.bold());
        if let Some(count) = tag.note_count {
            println!("    {} articles", count.to_string().dimmed());
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
        format!("Searching for tags matching '{}'...", keyword).cyan()
    );

    let hashtags = client.search_hashtags(keyword).await?;

    if hashtags.is_empty() {
        println!("{}", "No matching hashtags found.".yellow());
        return Ok(());
    }

    println!("\n{} matching hashtags:\n", hashtags.len());

    for tag in hashtags {
        println!("  {} {}", "#".dimmed(), tag.name.green());
        if let Some(count) = tag.note_count {
            println!("    {} articles", count.to_string().dimmed());
        }
    }

    Ok(())
}
