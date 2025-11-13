use crate::auth::Credentials;
use crate::error::Result;
use colored::Colorize;
use dialoguer::Input;

pub async fn login() -> Result<()> {
    println!("{}", "Note Authentication".bold());
    println!("\nTo authenticate:");
    println!("1. Open https://note.com and log in");
    println!("2. Open browser DevTools (F12) → Application/Storage → Cookies");
    println!("3. Copy the value of '_note_session_v5' cookie\n");

    let session_cookie: String = Input::new()
        .with_prompt("Enter your Note session cookie")
        .interact_text()?;

    let csrf_token: String = Input::new()
        .with_prompt("Enter CSRF token (optional, press Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let csrf = if csrf_token.is_empty() {
        None
    } else {
        Some(csrf_token)
    };

    let credentials = Credentials::new(session_cookie, csrf);
    credentials.save()?;

    println!("{}", "\n✓ Authentication successful!".green());
    println!("Credentials saved securely in system keyring.");

    Ok(())
}

pub async fn status() -> Result<()> {
    if Credentials::exists() {
        let credentials = Credentials::load()?;
        println!("{}", "✓ Authenticated".green());
        println!(
            "\nSession cookie: {}",
            mask_cookie(&credentials.session_cookie)
        );

        if let Some(ref csrf) = credentials.csrf_token {
            println!("CSRF token: {}", mask_token(csrf));
        }
    } else {
        println!("{}", "✗ Not authenticated".red());
        println!("Run 'noet auth login' to authenticate.");
    }

    Ok(())
}

pub async fn refresh() -> Result<()> {
    if !Credentials::exists() {
        println!("{}", "✗ Not authenticated".red());
        println!("Run 'noet auth login' to authenticate first.");
        return Ok(());
    }

    println!("{}", "Refreshing authentication...".cyan());
    login().await
}

pub async fn clear() -> Result<()> {
    if !Credentials::exists() {
        println!("{}", "No credentials found.".yellow());
        return Ok(());
    }

    Credentials::delete()?;
    println!("{}", "✓ Credentials cleared.".green());

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
