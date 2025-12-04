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

/// Parse editor command handling quoted arguments
fn parse_editor_command(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in cmd.chars() {
        match c {
            '"' | '\'' => {
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(c);
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

/// Open a file in the configured editor
pub fn open_in_editor<P: AsRef<Path>>(filepath: P) -> Result<()> {
    let editor_cmd = get_editor()?;
    let parts = parse_editor_command(&editor_cmd);

    if parts.is_empty() {
        return Err(NoetError::ConfigError(
            "エディタコマンドが空です".to_string(),
        ));
    }

    let editor = &parts[0];
    let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

    let mut command = Command::new(editor);
    command.args(args);
    command.arg(filepath.as_ref());

    let status = command.status().map_err(|e| {
        NoetError::ConfigError(format!("エディタ '{editor}' の起動に失敗しました: {e}"))
    })?;

    if !status.success() {
        return Err(NoetError::ConfigError(format!(
            "エディタが異常終了しました: {status}"
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

    #[test]
    fn test_parse_editor_command_simple() {
        let cmd = "vim";
        let parts = parse_editor_command(cmd);
        assert_eq!(parts, vec!["vim"]);
    }

    #[test]
    fn test_parse_editor_command_with_args() {
        let cmd = "code -w -n";
        let parts = parse_editor_command(cmd);
        assert_eq!(parts, vec!["code", "-w", "-n"]);
    }

    #[test]
    fn test_parse_editor_command_with_double_quotes() {
        let cmd = r#"code -w "/path/with spaces/file.txt""#;
        let parts = parse_editor_command(cmd);
        assert_eq!(parts, vec!["code", "-w", "/path/with spaces/file.txt"]);
    }

    #[test]
    fn test_parse_editor_command_with_single_quotes() {
        let cmd = r#"vim '+set number' -c 'echo hello'"#;
        let parts = parse_editor_command(cmd);
        assert_eq!(parts, vec!["vim", "+set number", "-c", "echo hello"]);
    }

    #[test]
    fn test_parse_editor_command_empty() {
        let cmd = "";
        let parts = parse_editor_command(cmd);
        assert_eq!(parts, Vec::<String>::new());
    }

    #[test]
    fn test_parse_editor_command_multiple_spaces() {
        let cmd = "code  -w   -n";
        let parts = parse_editor_command(cmd);
        assert_eq!(parts, vec!["code", "-w", "-n"]);
    }
}
