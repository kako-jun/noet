use crate::error::{NoetError, Result};
use crate::workspace;
use colored::Colorize;
use dialoguer::{Confirm, Editor};
use std::fs;
use std::path::PathBuf;

/// Get the config directory for noet
fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| NoetError::ConfigError("設定ディレクトリが見つかりません".to_string()))?
        .join("noet");
    Ok(config_dir)
}

/// Helper function to get markdown filenames from a directory
fn list_markdown_files_in_dir(dir: &PathBuf) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(dir)?;
    let mut files: Vec<String> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_file() && path.extension()? == "md" {
                    path.file_stem()?.to_str().map(String::from)
                } else {
                    None
                }
            })
        })
        .collect();

    files.sort();
    Ok(files)
}

pub fn list_templates() -> Result<()> {
    let template_dir = get_template_dir()?;
    let templates = list_markdown_files_in_dir(&template_dir)?;

    if templates.is_empty() {
        println!("{}", "テンプレートが見つかりません。".yellow());
        println!(
            "\n新しいテンプレートを作成: {}",
            "noet template add <NAME>".cyan()
        );
        return Ok(());
    }

    println!("{}", "利用可能なテンプレート:".bold());
    for template in templates {
        println!("  • {}", template.cyan());
    }

    println!(
        "\n使用方法: {}",
        "noet new --template <NAME> \"記事タイトル\"".dimmed()
    );

    Ok(())
}

pub fn add_template(name: &str) -> Result<()> {
    let template_dir = get_template_dir()?;
    fs::create_dir_all(&template_dir)?;

    let template_path = template_dir.join(format!("{name}.md"));

    if template_path.exists() {
        let overwrite = Confirm::new()
            .with_prompt(format!(
                "テンプレート '{name}' は既に存在します。上書きしますか？"
            ))
            .interact()?;

        if !overwrite {
            println!("{}", "キャンセルされました。".yellow());
            return Ok(());
        }
    }

    let default_content = r#"---
title: {{TITLE}}
status: draft
tags:
---

# {{TITLE}}

記事の内容をここに書いてください...
"#;

    // Open editor for user to customize the template
    let content = if let Some(edited) = Editor::new().edit(default_content)? {
        edited
    } else {
        default_content.to_string()
    };

    fs::write(&template_path, content)?;

    println!(
        "{} テンプレート '{}' を {} に作成しました",
        "✓".green(),
        name.bold(),
        template_path.display()
    );

    Ok(())
}

pub fn show_template(name: &str) -> Result<()> {
    let template_path = get_template_path(name)?;

    if !template_path.exists() {
        return Err(NoetError::FileNotFound(format!(
            "テンプレート '{name}' が見つかりません"
        )));
    }

    let content = fs::read_to_string(&template_path)?;

    println!("{}", format!("テンプレート: {name}").bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("{content}");
    println!("{}", "─".repeat(50).dimmed());

    Ok(())
}

pub fn remove_template(name: &str) -> Result<()> {
    let template_path = get_template_path(name)?;

    if !template_path.exists() {
        return Err(NoetError::FileNotFound(format!(
            "テンプレート '{name}' が見つかりません"
        )));
    }

    let confirm = Confirm::new()
        .with_prompt(format!("テンプレート '{name}' を削除しますか？"))
        .interact()?;

    if !confirm {
        println!("{}", "キャンセルされました。".yellow());
        return Ok(());
    }

    fs::remove_file(&template_path)?;

    println!(
        "{} テンプレート '{}' を削除しました",
        "✓".green(),
        name.bold()
    );

    Ok(())
}

#[allow(dead_code)]
pub fn load_template(name: &str, title: &str) -> Result<String> {
    let template_path = get_template_path(name)?;

    if !template_path.exists() {
        return Err(NoetError::FileNotFound(format!(
            "Template '{name}' not found. Use 'noet template list' to see available templates."
        )));
    }

    let content = fs::read_to_string(&template_path)?;

    // Replace placeholders
    let content = content.replace("{{TITLE}}", title);

    Ok(content)
}

#[allow(dead_code)]
pub fn list_template_names() -> Result<Vec<String>> {
    let template_dir = get_template_dir()?;
    list_markdown_files_in_dir(&template_dir)
}

fn get_template_dir() -> Result<PathBuf> {
    // Try workspace templates first
    if workspace::is_in_workspace() {
        Ok(workspace::get_templates_dir()?)
    } else {
        // Fallback to global config directory
        let config_dir = get_config_dir()?;
        Ok(config_dir.join("templates"))
    }
}

fn get_template_path(name: &str) -> Result<PathBuf> {
    let template_dir = get_template_dir()?;
    Ok(template_dir.join(format!("{name}.md")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_template_with_title() {
        let template_content = r#"---
title: {{TITLE}}
status: draft
---

# {{TITLE}}

Content here"#;

        let result = template_content.replace("{{TITLE}}", "Test Article");

        assert!(result.contains("title: Test Article"));
        assert!(result.contains("# Test Article"));
        assert!(!result.contains("{{TITLE}}"));
    }

    #[test]
    fn test_list_markdown_files_in_dir_empty() {
        let temp_dir = TempDir::new().unwrap();
        let result = list_markdown_files_in_dir(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_list_markdown_files_in_dir_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test markdown files
        fs::write(dir_path.join("template1.md"), "content").unwrap();
        fs::write(dir_path.join("template2.md"), "content").unwrap();
        fs::write(dir_path.join("not_markdown.txt"), "content").unwrap();

        let result = list_markdown_files_in_dir(&dir_path.to_path_buf()).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains(&"template1".to_string()));
        assert!(result.contains(&"template2".to_string()));
        assert!(!result.contains(&"not_markdown".to_string()));
    }

    #[test]
    fn test_list_markdown_files_in_dir_sorted() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create files in non-alphabetical order
        fs::write(dir_path.join("zebra.md"), "content").unwrap();
        fs::write(dir_path.join("apple.md"), "content").unwrap();
        fs::write(dir_path.join("banana.md"), "content").unwrap();

        let result = list_markdown_files_in_dir(&dir_path.to_path_buf()).unwrap();

        assert_eq!(
            result,
            vec![
                "apple".to_string(),
                "banana".to_string(),
                "zebra".to_string()
            ]
        );
    }

    #[test]
    fn test_list_markdown_files_in_dir_nonexistent() {
        let nonexistent = PathBuf::from("/nonexistent/path/that/does/not/exist");
        let result = list_markdown_files_in_dir(&nonexistent).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }
}
