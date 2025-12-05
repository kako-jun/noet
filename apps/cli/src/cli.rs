use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "noet")]
#[command(about = "Note.com CLI - ブラウザ拡張機能経由で記事を管理", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Run as Native Messaging host for browser extension
    #[arg(long, hide = true)]
    pub native_messaging: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Setup browser extension (download, install guide, configure native messaging)
    Setup,

    /// Initialize a noet working directory
    Init {
        /// Directory path (default: current directory)
        path: Option<PathBuf>,
    },

    /// Check connection to browser extension
    Ping,

    /// Check authentication status (Note.com login)
    Auth,

    /// List your articles
    List,

    /// Get article content
    Get {
        /// Username
        #[arg(short, long)]
        username: String,

        /// Article key
        key: String,
    },

    /// Create a new article from markdown file
    Create {
        /// Path to the markdown file
        file: PathBuf,

        /// Save as draft instead of publishing
        #[arg(short, long)]
        draft: bool,
    },

    /// Update an existing article
    Update {
        /// Article key
        key: String,

        /// Path to the markdown file
        file: PathBuf,

        /// Save as draft instead of publishing
        #[arg(short, long)]
        draft: bool,
    },

    /// Delete an article
    Delete {
        /// Article key
        key: String,
    },

    /// Template management commands
    #[command(subcommand)]
    Template(TemplateCommands),
}

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// List all templates
    List,

    /// Create a new template
    Add {
        /// Template name
        name: String,
    },

    /// Show template content
    Show {
        /// Template name
        name: String,
    },

    /// Remove a template
    Remove {
        /// Template name
        name: String,
    },
}
