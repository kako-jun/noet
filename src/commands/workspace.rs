use crate::error::Result;
use crate::workspace as ws;
use colored::Colorize;
use std::path::PathBuf;

pub async fn init(path: Option<PathBuf>) -> Result<()> {
    let workspace_root = ws::init_workspace(path)?;

    println!(
        "{} Initialized noet workspace at {}",
        "✓".green().bold(),
        workspace_root.display().to_string().cyan()
    );

    println!("\nCreated:");
    println!("  • {} - Configuration directory", ".noet/".dimmed());
    println!("  • {} - Templates directory", "templates/".dimmed());
    println!("  • {} - Updated gitignore", ".gitignore".dimmed());

    println!("\n{}", "Next steps:".bold());
    println!(
        "  1. Create a new article: {}",
        "noet new \"My Article\"".cyan()
    );
    println!(
        "  2. Create a template: {}",
        "noet template add my-template".cyan()
    );
    println!(
        "  3. Export articles: {}",
        "noet export --all --username <USER>".cyan()
    );

    Ok(())
}
