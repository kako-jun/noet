mod api;
mod auth;
mod cli;
mod commands;
mod config;
mod error;
mod models;

use clap::Parser;
use cli::{AuthCommands, Cli, Commands, MagazineCommands, TagCommands};
use colored::Colorize;

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(e) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> error::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { title } => {
            commands::article::new_article(title).await?;
        }

        Commands::Publish { file, draft } => {
            commands::article::publish_article(&file, draft).await?;
        }

        Commands::Edit { id, file } => {
            commands::article::edit_article(&id, &file).await?;
        }

        Commands::Delete { id, force } => {
            commands::article::delete_article(&id, force).await?;
        }

        Commands::List { username, page } => {
            commands::article::list_articles(&username, page).await?;
        }

        Commands::Tag(tag_cmd) => match tag_cmd {
            TagCommands::List { page } => {
                commands::tag::list_tags(page).await?;
            }
            TagCommands::Suggest { keyword } => {
                commands::tag::suggest_tags(&keyword).await?;
            }
        },

        Commands::Magazine(mag_cmd) => match mag_cmd {
            MagazineCommands::Add {
                magazine,
                note_id,
                note_key,
            } => {
                commands::magazine::add_to_magazine(&magazine, &note_id, &note_key).await?;
            }
            MagazineCommands::Remove { magazine, note_key } => {
                commands::magazine::remove_from_magazine(&magazine, &note_key).await?;
            }
        },

        Commands::Like { key } => {
            commands::engagement::like_article(&key).await?;
        }

        Commands::Unlike { key } => {
            commands::engagement::unlike_article(&key).await?;
        }

        Commands::Comments { id } => {
            commands::engagement::show_comments(&id).await?;
        }

        Commands::User { username } => {
            commands::user::show_user_info(&username).await?;
        }

        Commands::Auth(auth_cmd) => match auth_cmd {
            AuthCommands::Login => {
                commands::auth::login().await?;
            }
            AuthCommands::Status => {
                commands::auth::status().await?;
            }
            AuthCommands::Refresh => {
                commands::auth::refresh().await?;
            }
            AuthCommands::Clear => {
                commands::auth::clear().await?;
            }
        },
    }

    Ok(())
}
