use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;

pub async fn show_user_info(username: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    println!("{}", "Fetching user information...".cyan());

    let user = client.get_user(username).await?;

    println!("\n{}", user.nickname.bold());
    println!("{} @{}", "Username:".dimmed(), user.urlname);

    if let Some(follower_count) = user.follower_count {
        println!("{} {}", "Followers:".dimmed(), follower_count);
    }

    if let Some(following_count) = user.following_count {
        println!("{} {}", "Following:".dimmed(), following_count);
    }

    Ok(())
}
