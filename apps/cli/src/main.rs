mod api;
mod auth;
mod cli;
mod commands;
mod config;
mod converters;
mod editor;
mod error;
mod models;
mod native_messaging;
mod rate_limiter;
mod tui_diff;
mod workspace;

use clap::Parser;
use cli::{AuthCommands, Cli, Commands, MagazineCommands, TagCommands, TemplateCommands};
use colored::Colorize;

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(e) = run().await {
        eprintln!("{} {}", "エラー:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> error::Result<()> {
    let cli = Cli::parse();

    // Run as Native Messaging host if flag is set
    if cli.native_messaging {
        return native_messaging::run().await;
    }

    // If no command provided, run interactive mode
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            commands::interactive::run_interactive_mode().await?;
            return Ok(());
        }
    };

    match command {
        Commands::Setup => {
            commands::setup::run_setup().await?;
        }

        Commands::Init { path } => {
            commands::workspace::init(path).await?;
        }

        Commands::New { title, template } => {
            commands::article::new_article(title, template).await?;
        }

        Commands::Publish { file, draft, force } => {
            commands::article::publish_article(&file, draft, force).await?;
        }

        Commands::Diff { file } => {
            commands::article::show_diff(&file).await?;
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

        Commands::Export {
            article_key,
            all,
            username,
            output,
            page,
        } => {
            commands::export::export_articles(article_key, all, username, output, page).await?;
        }

        Commands::Template(template_cmd) => match template_cmd {
            TemplateCommands::List => {
                commands::template::list_templates()?;
            }
            TemplateCommands::Add { name } => {
                commands::template::add_template(&name)?;
            }
            TemplateCommands::Show { name } => {
                commands::template::show_template(&name)?;
            }
            TemplateCommands::Remove { name } => {
                commands::template::remove_template(&name)?;
            }
        },

        Commands::Auth(auth_cmd) => match auth_cmd {
            AuthCommands::Status => {
                commands::auth::status().await?;
            }
        },
    }

    Ok(())
}
