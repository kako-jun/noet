use crate::config::Config;
use crate::error::{NoetError, Result};
use colored::Colorize;
use dialoguer::{Confirm, Editor};
use std::fs;
use std::path::PathBuf;

pub async fn list_templates() -> Result<()> {
    let template_dir = get_template_dir()?;

    if !template_dir.exists() {
        println!("{}", "No templates found.".yellow());
        println!(
            "\nCreate a new template with: {}",
            "noet template add <NAME>".cyan()
        );
        return Ok(());
    }

    let entries = fs::read_dir(&template_dir)?;
    let mut templates: Vec<String> = entries
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

    if templates.is_empty() {
        println!("{}", "No templates found.".yellow());
        return Ok(());
    }

    templates.sort();

    println!("{}", "Available templates:".bold());
    for template in templates {
        println!("  • {}", template.cyan());
    }

    println!(
        "\nUse with: {}",
        "noet new --template <NAME> \"Article Title\"".dimmed()
    );

    Ok(())
}

pub async fn add_template(name: &str) -> Result<()> {
    let template_dir = get_template_dir()?;
    fs::create_dir_all(&template_dir)?;

    let template_path = template_dir.join(format!("{}.md", name));

    if template_path.exists() {
        let overwrite = Confirm::new()
            .with_prompt(format!("Template '{}' already exists. Overwrite?", name))
            .interact()?;

        if !overwrite {
            println!("{}", "Cancelled.".yellow());
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
        "{} Template '{}' created at {}",
        "✓".green(),
        name.bold(),
        template_path.display()
    );

    Ok(())
}

pub async fn show_template(name: &str) -> Result<()> {
    let template_path = get_template_path(name)?;

    if !template_path.exists() {
        return Err(NoetError::FileNotFound(format!(
            "Template '{}' not found",
            name
        )));
    }

    let content = fs::read_to_string(&template_path)?;

    println!("{}", format!("Template: {}", name).bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", content);
    println!("{}", "─".repeat(50).dimmed());

    Ok(())
}

pub async fn remove_template(name: &str) -> Result<()> {
    let template_path = get_template_path(name)?;

    if !template_path.exists() {
        return Err(NoetError::FileNotFound(format!(
            "Template '{}' not found",
            name
        )));
    }

    let confirm = Confirm::new()
        .with_prompt(format!("Remove template '{}'?", name))
        .interact()?;

    if !confirm {
        println!("{}", "Cancelled.".yellow());
        return Ok(());
    }

    fs::remove_file(&template_path)?;

    println!("{} Template '{}' removed", "✓".green(), name.bold());

    Ok(())
}

pub fn load_template(name: &str, title: &str) -> Result<String> {
    let template_path = get_template_path(name)?;

    if !template_path.exists() {
        return Err(NoetError::FileNotFound(format!(
            "Template '{}' not found. Use 'noet template list' to see available templates.",
            name
        )));
    }

    let content = fs::read_to_string(&template_path)?;

    // Replace placeholders
    let content = content.replace("{{TITLE}}", title);

    Ok(content)
}

fn get_template_dir() -> Result<PathBuf> {
    let config_dir = Config::config_dir()?;
    Ok(config_dir.join("templates"))
}

fn get_template_path(name: &str) -> Result<PathBuf> {
    let template_dir = get_template_dir()?;
    Ok(template_dir.join(format!("{}.md", name)))
}

#[cfg(test)]
mod tests {
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
}
