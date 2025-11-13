use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;

pub async fn add_to_magazine(magazine_key: &str, note_id: &str, note_key: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "Adding article to magazine...".cyan());

    client.add_to_magazine(magazine_key, note_id, note_key).await?;

    println!(
        "{} Article added to magazine '{}'",
        "✓".green(),
        magazine_key
    );

    Ok(())
}

pub async fn remove_from_magazine(magazine_key: &str, note_key: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "Removing article from magazine...".cyan());

    client.remove_from_magazine(magazine_key, note_key).await?;

    println!(
        "{} Article removed from magazine '{}'",
        "✓".green(),
        magazine_key
    );

    Ok(())
}
