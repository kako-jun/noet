use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;

pub async fn add_to_magazine(magazine_key: &str, note_id: &str, note_key: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "マガジンに記事を追加中...".cyan());

    client
        .add_to_magazine(magazine_key, note_id, note_key)
        .await?;

    println!(
        "{} マガジン '{}' に記事を追加しました",
        "✓".green(),
        magazine_key
    );

    Ok(())
}

pub async fn remove_from_magazine(magazine_key: &str, note_key: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "マガジンから記事を削除中...".cyan());

    client.remove_from_magazine(magazine_key, note_key).await?;

    println!(
        "{} マガジン '{}' から記事を削除しました",
        "✓".green(),
        magazine_key
    );

    Ok(())
}
