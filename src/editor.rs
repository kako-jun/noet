use crate::config::Config;
use crate::error::{NoetError, Result};
use std::env;
use std::path::Path;
use std::process::Command;

/// Get the editor command from config, environment variables, or platform default
pub fn get_editor() -> Result<String> {
    // 1. Check config file
    if let Ok(config) = Config::load() {
        if let Some(editor) = config.editor {
            return Ok(editor);
        }
    }

    // 2. Check environment variables
    if let Ok(editor) = env::var("VISUAL") {
        return Ok(editor);
    }
    if let Ok(editor) = env::var("EDITOR") {
        return Ok(editor);
    }

    // 3. Platform defaults
    #[cfg(target_os = "windows")]
    return Ok("notepad".to_string());

    #[cfg(target_os = "macos")]
    return Ok("open -e".to_string());

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return Ok("vim".to_string());
}

/// Open a file in the configured editor
pub fn open_in_editor<P: AsRef<Path>>(filepath: P) -> Result<()> {
    let editor_cmd = get_editor()?;
    let parts: Vec<&str> = editor_cmd.split_whitespace().collect();

    if parts.is_empty() {
        return Err(NoetError::ConfigError(
            "Editor command is empty".to_string(),
        ));
    }

    let editor = parts[0];
    let args: Vec<&str> = parts[1..].to_vec();

    let mut command = Command::new(editor);
    command.args(args);
    command.arg(filepath.as_ref());

    let status = command.status().map_err(|e| {
        NoetError::ConfigError(format!("Failed to launch editor '{}': {}", editor, e))
    })?;

    if !status.success() {
        return Err(NoetError::ConfigError(format!(
            "Editor exited with non-zero status: {}",
            status
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_editor_has_default() {
        // Should always return something (either from config, env, or default)
        let editor = get_editor();
        assert!(editor.is_ok());
        assert!(!editor.unwrap().is_empty());
    }
}
