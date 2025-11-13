use crate::auth::Credentials;
use crate::error::Result;
use colored::Colorize;
use dialoguer::Input;

pub async fn login() -> Result<()> {
    println!("{}", "Note 認証".bold());
    println!("\n認証するには:");
    println!("1. https://note.com にログイン");
    println!("2. ブラウザの開発者ツール (F12) → Application/Storage → Cookies");
    println!("3. '_note_session_v5' クッキーの値をコピー\n");

    let session_cookie: String = Input::new()
        .with_prompt("Note セッションクッキーを入力")
        .interact_text()?;

    let xsrf_token: String = Input::new()
        .with_prompt("XSRF トークンを入力 (オプション、スキップする場合は Enter)")
        .allow_empty(true)
        .interact_text()?;

    let xsrf = if xsrf_token.is_empty() {
        None
    } else {
        Some(xsrf_token)
    };

    let credentials = Credentials::new(session_cookie, xsrf);
    credentials.save()?;

    println!("{}", "\n✓ 認証に成功しました！".green());
    println!("認証情報はシステムキーリングに安全に保存されました。");

    Ok(())
}

pub async fn status() -> Result<()> {
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
        println!("'noet auth login' を実行して認証してください。");
    }

    Ok(())
}

pub async fn refresh() -> Result<()> {
    if !Credentials::exists() {
        println!("{}", "✗ 未認証".red());
        println!("先に 'noet auth login' を実行して認証してください。");
        return Ok(());
    }

    println!("{}", "認証情報を更新しています...".cyan());
    login().await
}

pub async fn clear() -> Result<()> {
    if !Credentials::exists() {
        println!("{}", "認証情報が見つかりません。".yellow());
        return Ok(());
    }

    Credentials::delete()?;
    println!("{}", "✓ 認証情報をクリアしました。".green());

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
