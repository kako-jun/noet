use crate::error::Result;
use crate::workspace as ws;
use colored::Colorize;
use std::path::PathBuf;

pub async fn init(path: Option<PathBuf>) -> Result<()> {
    let workspace_root = ws::init_workspace(path)?;

    println!(
        "{} {} に noet ワークスペースを初期化しました",
        "✓".green().bold(),
        workspace_root.display().to_string().cyan()
    );

    println!("\n作成されたファイル:");
    println!("  • {} - 設定ディレクトリ", ".noet/".dimmed());
    println!("  • {} - テンプレートディレクトリ", "templates/".dimmed());
    println!("  • {} - .gitignore を更新", ".gitignore".dimmed());

    println!("\n{}", "次のステップ:".bold());
    println!("  1. 新規記事を作成: {}", "noet new \"My Article\"".cyan());
    println!(
        "  2. テンプレートを作成: {}",
        "noet template add my-template".cyan()
    );
    println!(
        "  3. 記事をエクスポート: {}",
        "noet export --all --username <USER>".cyan()
    );

    Ok(())
}
