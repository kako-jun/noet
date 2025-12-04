use crate::auth::Credentials;
use crate::error::Result;
use colored::Colorize;

pub async fn status() -> Result<()> {
    println!("{}", "=== Note認証状態 ===".bold());
    println!();
    println!("認証は環境変数で管理されます：");
    println!("  NOET_SESSION_COOKIE - Note.comのセッションクッキー（必須）");
    println!("  NOET_XSRF_TOKEN     - XSRFトークン（オプション）");
    println!();

    if Credentials::exists() {
        let credentials = Credentials::load()?;
        println!("{}", "✓ 認証済み".green());
        println!(
            "\nセッションクッキー: {}",
            mask_cookie(&credentials.session_cookie)
        );

        if let Some(ref xsrf) = credentials.xsrf_token {
            println!("XSRF トークン: {}", mask_token(xsrf));
        }
    } else {
        println!("{}", "✗ 未認証".red());
        println!();
        println!("{}", "セットアップ手順:".bold());
        println!();
        println!("1. Note.comのクッキーを取得:");
        println!("   • https://note.com にログイン");
        println!("   • ブラウザの開発者ツール (F12) → Application/Storage → Cookies");
        println!("   • '_note_session_v5' の値をコピー");
        println!();
        println!("2. 環境変数を設定（シェル設定ファイルに追加して永続化）:");
        println!();
        println!(
            "   {}:",
            "Bash/Zsh (Linux: ~/.bashrc, ~/.zshrc / macOS: ~/.bash_profile, ~/.zshrc)".cyan()
        );
        println!("   export NOET_SESSION_COOKIE=\"your_cookie_value\"");
        println!("   export NOET_XSRF_TOKEN=\"your_xsrf_token\"  # オプション");
        println!();
        println!("   {}:", "Fish (~/.config/fish/config.fish)".cyan());
        println!("   set -x NOET_SESSION_COOKIE \"your_cookie_value\"");
        println!("   set -x NOET_XSRF_TOKEN \"your_xsrf_token\"  # オプション");
        println!();
        println!("   {}:", "PowerShell (プロフィール)".cyan());
        println!("   $env:NOET_SESSION_COOKIE = \"your_cookie_value\"");
        println!("   $env:NOET_XSRF_TOKEN = \"your_xsrf_token\"  # オプション");
        println!();
        println!("   {}:", "Windows CMD (システム環境変数)".cyan());
        println!("   setx NOET_SESSION_COOKIE \"your_cookie_value\"");
        println!("   setx NOET_XSRF_TOKEN \"your_xsrf_token\"  # オプション");
        println!();
        println!("3. シェルを再起動するか、設定をリロード:");
        println!("   source ~/.zshrc   # Zsh (macOS標準/Linux)");
        println!("   source ~/.bashrc  # Bash (Linux)");
        println!("   source ~/.bash_profile  # Bash (macOS)");
        println!("   新しいターミナルウィンドウを開く  # Windows");
        println!();
        println!(
            "{} 一時的に使う場合は、コマンド実行時に指定:",
            "TIP:".yellow()
        );
        println!("   NOET_SESSION_COOKIE=\"...\" noet list username  # Unix系");
        println!("   $env:NOET_SESSION_COOKIE=\"...\"; noet list username  # PowerShell");
    }

    Ok(())
}

fn mask_cookie(cookie: &str) -> String {
    if cookie.len() > 20 {
        format!("{}...{}", &cookie[..10], &cookie[cookie.len() - 10..])
    } else {
        "***".to_string()
    }
}

fn mask_token(token: &str) -> String {
    if token.len() > 16 {
        format!("{}...{}", &token[..8], &token[token.len() - 8..])
    } else {
        "***".to_string()
    }
}
