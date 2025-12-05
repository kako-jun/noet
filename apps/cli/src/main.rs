mod cli;
mod commands;
mod converters;
mod editor;
mod error;
mod extension_client;
mod image_handler;
mod native_messaging;
mod workspace;

use clap::Parser;
use cli::{Cli, Commands, TemplateCommands};
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

    // If no command provided, show help
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            println!("noet - Note.com CLI (ブラウザ拡張機能経由)");
            println!();
            println!("使い方: noet <COMMAND>");
            println!();
            println!("まず `noet setup` を実行して拡張機能をインストールしてください。");
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

        Commands::Ping => {
            commands::extension::ping().await?;
        }

        Commands::Auth => {
            commands::extension::check_auth().await?;
        }

        Commands::List => {
            commands::extension::list_articles().await?;
        }

        Commands::Get {
            username,
            key,
            save,
        } => {
            commands::extension::get_article(&username, &key, save.as_deref()).await?;
        }

        Commands::Create { file, draft } => {
            commands::extension::create_article(&file, draft).await?;
        }

        Commands::Update { key, file, draft } => {
            commands::extension::update_article(&key, &file, draft).await?;
        }

        Commands::Delete { key } => {
            commands::extension::delete_article(&key).await?;
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
    }

    Ok(())
}
