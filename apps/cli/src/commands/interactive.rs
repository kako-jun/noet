use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::commands::{article, template};
use crate::config::Config;
use crate::editor;
use crate::error::Result;
use crate::workspace;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use dialoguer::Select;
use std::env;
use std::fs;

pub async fn run_interactive_mode() -> Result<()> {
    println!("{}", "noet".cyan().bold());
    println!();

    loop {
        let options = [
            "📝 [n] 新規記事を作成",
            "✏️  [e] 既存記事を編集",
            "📤 [p] 記事を公開",
            "📋 [l] 自分の記事一覧",
            "🚪 [q] 終了",
        ];

        println!("{}", "選択してください:".bold());
        for (i, option) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, option);
        }
        println!();
        print!("キー入力 (n/e/p/l/q または 1-5): ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let selection = read_menu_input()?;

        println!();

        match selection {
            0 => create_new_article().await?,
            1 => edit_existing_article().await?,
            2 => publish_article().await?,
            3 => list_my_articles().await?,
            4 => {
                println!("{}", "終了します".dimmed());
                break;
            }
            _ => {
                println!("{}", "無効な選択です".yellow());
                continue;
            }
        }

        println!();
    }

    Ok(())
}

fn read_menu_input() -> Result<usize> {
    enable_raw_mode()?;

    let selection = loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            let result = match code {
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('1') => Some(0),
                KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Char('2') => Some(1),
                KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char('3') => Some(2),
                KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Char('4') => Some(3),
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Char('5') => Some(4),
                KeyCode::Esc => Some(4),
                _ => None,
            };

            if let Some(selection) = result {
                break selection;
            }
        }
    };

    disable_raw_mode()?;
    println!();

    Ok(selection)
}

async fn create_new_article() -> Result<()> {
    // Check if in workspace
    if !workspace::is_in_workspace() {
        println!(
            "{} Not in a noet workspace. Run {} to initialize.",
            "Warning:".yellow(),
            "noet init".cyan()
        );
    }

    // Ask for title (optional)
    let title = dialoguer::Input::<String>::new()
        .with_prompt("記事タイトル (空欄可)")
        .allow_empty(true)
        .interact_text()?;

    let title = if title.trim().is_empty() {
        "untitled".to_string()
    } else {
        title
    };

    // Ask for template (optional)
    let use_template = dialoguer::Confirm::new()
        .with_prompt("テンプレートを使用しますか？")
        .default(false)
        .interact()?;

    let template_name = if use_template {
        // List available templates
        let templates = template::list_template_names()?;
        if templates.is_empty() {
            println!("{}", "利用可能なテンプレートがありません".yellow());
            None
        } else {
            let selection = Select::new()
                .with_prompt("テンプレートを選択")
                .items(&templates)
                .interact()?;
            Some(templates[selection].clone())
        }
    } else {
        None
    };

    // Create article using existing function
    article::new_article(Some(title.clone()), template_name).await?;

    // Generate filename
    let filename = title
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let current_dir = env::current_dir()?;
    let filepath = current_dir.join(format!("{filename}.md"));

    // Open in editor
    println!("{}", "エディタを起動します...".cyan());
    if let Err(e) = editor::open_in_editor(&filepath) {
        println!("{} {}", "Warning:".yellow(), e);
        println!(
            "{}",
            format!("手動で編集してください: {}", filepath.display()).dimmed()
        );
    }

    Ok(())
}

async fn edit_existing_article() -> Result<()> {
    // List markdown files in current directory
    let current_dir = env::current_dir()?;
    let entries = fs::read_dir(&current_dir)?;

    let mut md_files: Vec<String> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "md" {
                path.file_name()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    if md_files.is_empty() {
        println!(
            "{}",
            "カレントディレクトリにMarkdownファイルが見つかりません".yellow()
        );
        return Ok(());
    }

    md_files.sort();

    let selection = Select::new()
        .with_prompt("編集するファイルを選択")
        .items(&md_files)
        .interact()?;

    let filepath = current_dir.join(&md_files[selection]);

    println!("{}", "エディタを起動します...".cyan());
    if let Err(e) = editor::open_in_editor(&filepath) {
        println!("{} {}", "Warning:".yellow(), e);
        println!(
            "{}",
            format!("手動で編集してください: {}", filepath.display()).dimmed()
        );
    }

    Ok(())
}

async fn publish_article() -> Result<()> {
    // List markdown files in current directory
    let current_dir = env::current_dir()?;
    let entries = fs::read_dir(&current_dir)?;

    let mut md_files: Vec<String> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "md" {
                path.file_name()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    if md_files.is_empty() {
        println!(
            "{}",
            "カレントディレクトリにMarkdownファイルが見つかりません".yellow()
        );
        return Ok(());
    }

    md_files.sort();

    let selection = Select::new()
        .with_prompt("公開するファイルを選択")
        .items(&md_files)
        .interact()?;

    let filepath = current_dir.join(&md_files[selection]);

    // Ask for draft status
    let as_draft = dialoguer::Confirm::new()
        .with_prompt("下書きとして公開しますか？")
        .default(true)
        .interact()?;

    // Publish using existing function
    article::publish_article(&filepath, as_draft, false).await?;

    Ok(())
}

async fn list_my_articles() -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    // Get username from credentials
    let username = client.get_username()?;

    println!("{}", format!("{username}の記事を取得中...").cyan());

    article::list_articles(&username, 1).await?;

    Ok(())
}
