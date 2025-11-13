use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;

pub async fn like_article(note_key: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    client.like_article(note_key).await?;

    println!("{} 記事 '{}' にいいねしました", "✓".green(), note_key);

    Ok(())
}

pub async fn unlike_article(note_key: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    client.unlike_article(note_key).await?;

    println!("{} 記事 '{}' のいいねを解除しました", "✓".green(), note_key);

    Ok(())
}

pub async fn show_comments(note_id: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "コメントを取得中...".cyan());

    let comments = client.get_comments(note_id).await?;

    if comments.is_empty() {
        println!("{}", "コメントが見つかりませんでした。".yellow());
        return Ok(());
    }

    println!("\n{} 件のコメント:\n", comments.len());

    for comment in comments {
        println!("{}", comment.user.nickname.bold());
        println!("  {}", comment.body);
        println!(
            "  {}",
            comment
                .created_at
                .format("%Y-%m-%d %H:%M")
                .to_string()
                .dimmed()
        );
        println!();
    }

    Ok(())
}
