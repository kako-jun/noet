use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;

pub async fn show_user_info(username: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "ユーザー情報を取得中...".cyan());

    let user = client.get_user(username).await?;

    println!("\n{}", user.nickname.bold());
    println!("{} @{}", "ユーザー名:".dimmed(), user.urlname);

    if let Some(follower_count) = user.follower_count {
        println!("{} {}", "フォロワー:".dimmed(), follower_count);
    }

    if let Some(following_count) = user.following_count {
        println!("{} {}", "フォロー中:".dimmed(), following_count);
    }

    Ok(())
}
