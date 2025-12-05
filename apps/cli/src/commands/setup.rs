use crate::error::{NoetError, Result};
use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Extension download URL (GitHub releases)
const EXTENSION_RELEASE_URL: &str =
    "https://github.com/kako-jun/noet/releases/latest/download/noet-extension.zip";

/// Local extension path (for development)
fn get_local_extension_path() -> Option<PathBuf> {
    // Check if we're running from the repo
    let exe_path = std::env::current_exe().ok()?;
    let repo_root = exe_path.ancestors().nth(3)?; // target/debug/noet -> repo root
    let local_ext = repo_root.join("apps").join("extension");
    if local_ext.join("manifest.json").exists() {
        Some(local_ext)
    } else {
        None
    }
}

/// Get the extension installation directory
/// - Linux: ~/.config/noet/extension
/// - macOS: ~/Library/Application Support/noet/extension
/// - Windows: %APPDATA%\noet\extension
fn get_extension_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| NoetError::ConfigError("設定ディレクトリが見つかりません".into()))?;
    Ok(config_dir.join("noet").join("extension"))
}

/// Get the native messaging manifest directory for the current platform
fn get_native_manifest_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir()
            .ok_or_else(|| NoetError::ConfigError("ホームディレクトリが見つかりません".into()))?;
        Ok(home
            .join(".config")
            .join("google-chrome")
            .join("NativeMessagingHosts"))
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir()
            .ok_or_else(|| NoetError::ConfigError("ホームディレクトリが見つかりません".into()))?;
        Ok(home
            .join("Library")
            .join("Application Support")
            .join("Google")
            .join("Chrome")
            .join("NativeMessagingHosts"))
    }

    #[cfg(target_os = "windows")]
    {
        // Windows uses registry, but we can also use a manifest file in AppData
        let app_data = dirs::config_dir()
            .ok_or_else(|| NoetError::ConfigError("AppDataディレクトリが見つかりません".into()))?;
        Ok(app_data.join("noet").join("NativeMessagingHosts"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(NoetError::ConfigError(
            "サポートされていないプラットフォームです".into(),
        ))
    }
}

/// Download and extract the extension
async fn download_extension(extension_dir: &PathBuf) -> Result<()> {
    println!("{}", "[1/4] 拡張機能をダウンロード中...".cyan());

    // Create extension directory
    fs::create_dir_all(extension_dir)?;

    let zip_path = extension_dir.join("noet-extension.zip");

    // Download using reqwest
    let response = reqwest::get(EXTENSION_RELEASE_URL)
        .await
        .map_err(|e| NoetError::Network(format!("ダウンロードに失敗しました: {e}")))?;

    if !response.status().is_success() {
        return Err(NoetError::Network(format!(
            "ダウンロードに失敗しました: HTTP {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| NoetError::Network(format!("データの読み込みに失敗しました: {e}")))?;

    fs::write(&zip_path, &bytes)?;

    println!("{}", "[2/4] 解凍中...".cyan());

    // Extract zip
    let file = fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| NoetError::ConfigError(format!("ZIPファイルの読み込みに失敗しました: {e}")))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            NoetError::ConfigError(format!("ZIPエントリの読み込みに失敗しました: {e}"))
        })?;

        let outpath = match file.enclosed_name() {
            Some(path) => extension_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    // Remove zip file
    fs::remove_file(&zip_path)?;

    println!("      {} {}", "解凍完了:".green(), extension_dir.display());

    Ok(())
}

/// Configure native messaging host
fn configure_native_messaging(_extension_dir: &Path) -> Result<()> {
    println!("{}", "[3/4] Native Messaging を設定中...".cyan());

    let manifest_dir = get_native_manifest_dir()?;
    fs::create_dir_all(&manifest_dir)?;

    // Get the path to the noet executable
    let exe_path = std::env::current_exe()?;

    // Create the native messaging manifest
    let manifest = serde_json::json!({
        "name": "com.noet.host",
        "description": "noet Native Messaging Host",
        "path": exe_path.to_string_lossy(),
        "type": "stdio",
        "allowed_origins": [
            "chrome-extension://noet-extension-id/"  // Will be updated after installation
        ]
    });

    let manifest_path = manifest_dir.join("com.noet.host.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, &manifest_json)?;

    println!(
        "      {} {}",
        "マニフェスト作成:".green(),
        manifest_path.display()
    );

    // For Windows, we need to update the registry
    #[cfg(target_os = "windows")]
    {
        println!(
            "      {} Windowsではレジストリの設定が必要です。",
            "注意:".yellow()
        );
        println!("      管理者権限で以下のコマンドを実行してください:");
        println!();
        println!(
            "      reg add \"HKCU\\Software\\Google\\Chrome\\NativeMessagingHosts\\com.noet.host\" /ve /t REG_SZ /d \"{}\" /f",
            manifest_path.display()
        );
    }

    Ok(())
}

/// Show Chrome extension installation instructions
fn show_installation_instructions(extension_dir: &Path) -> Result<()> {
    println!();
    println!("{}", "━".repeat(60).dimmed());
    println!();
    println!(
        "{}",
        "[4/4] Chrome に拡張機能をインストールしてください".cyan()
    );
    println!();
    println!("      以下の手順で拡張機能をインストールします:");
    println!();
    println!("      {}", "1. Chromeで chrome://extensions を開く".white());
    println!(
        "      {}",
        "2. 右上の「デベロッパーモード」をONにする".white()
    );
    println!(
        "      {}",
        "3. 「パッケージ化されていない拡張機能を読み込む」をクリック".white()
    );
    println!("      {}", "4. 以下のフォルダを選択:".white());
    println!();
    println!(
        "         {}",
        extension_dir.display().to_string().yellow().bold()
    );
    println!();
    println!("{}", "━".repeat(60).dimmed());
    println!();

    Ok(())
}

/// Wait for user to press Enter
fn wait_for_enter(prompt: &str) -> Result<()> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

/// Open Chrome extensions page
fn open_extensions_page() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg("chrome://extensions")
            .spawn();
        // xdg-open might not work with chrome:// URLs, try direct chrome
        let _ = std::process::Command::new("google-chrome")
            .arg("chrome://extensions")
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-a")
            .arg("Google Chrome")
            .arg("chrome://extensions")
            .spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "chrome://extensions"])
            .spawn();
    }

    Ok(())
}

/// Test connection to the extension
async fn test_extension_connection() -> Result<bool> {
    println!("{}", "接続テスト中...".cyan());

    // TODO: Implement actual connection test via native messaging
    // For now, we just check if the manifest exists
    let manifest_dir = get_native_manifest_dir()?;
    let manifest_path = manifest_dir.join("com.noet.host.json");

    if manifest_path.exists() {
        println!(
            "      {} Native Messaging マニフェストが設定されています",
            "✓".green()
        );
        return Ok(true);
    }

    Ok(false)
}

/// Run the setup wizard
pub async fn run_setup() -> Result<()> {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "║           noet セットアップウィザード                    ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝"
            .cyan()
            .bold()
    );
    println!();

    // Check for local development extension first
    let extension_dir = if let Some(local_path) = get_local_extension_path() {
        println!(
            "      {} 開発モード: ローカルの拡張機能を使用します",
            "⚙".cyan()
        );
        println!("      {}", local_path.display().to_string().dimmed());
        println!();
        local_path
    } else {
        let ext_dir = get_extension_dir()?;

        // Check if extension is already installed
        if ext_dir.exists() && ext_dir.join("manifest.json").exists() {
            println!(
                "      {} 拡張機能は既にダウンロード済みです: {}",
                "✓".green(),
                ext_dir.display()
            );
            println!();

            print!("      再ダウンロードしますか？ [y/N]: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!();
                println!("      既存の拡張機能を使用します。");
            } else {
                // Remove and re-download
                fs::remove_dir_all(&ext_dir)?;
                download_extension(&ext_dir).await?;
            }
        } else {
            // Download extension
            download_extension(&ext_dir).await?;
        }
        ext_dir
    };

    // Configure native messaging
    configure_native_messaging(&extension_dir)?;

    // Show installation instructions
    show_installation_instructions(&extension_dir)?;

    // Open Chrome extensions page
    print!("      Chromeの拡張機能ページを開きますか？ [Y/n]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "n" {
        open_extensions_page()?;
    }

    // Wait for user to install extension
    println!();
    wait_for_enter("      拡張機能をインストールしたらEnterを押してください...")?;

    // Test connection
    println!();
    let connected = test_extension_connection().await?;

    if connected {
        println!();
        println!(
            "{}",
            "╔══════════════════════════════════════════════════════════╗"
                .green()
                .bold()
        );
        println!(
            "{}",
            "║           セットアップ完了！                             ║"
                .green()
                .bold()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════╝"
                .green()
                .bold()
        );
        println!();
        println!("      次のステップ:");
        println!("      1. Note.com にログインしてください");
        println!("      2. `noet list <username>` で記事一覧を取得できます");
        println!();
    } else {
        println!();
        println!(
            "{}",
            "セットアップが完了していない可能性があります。".yellow()
        );
        println!("      拡張機能のインストールを確認してください。");
    }

    Ok(())
}
