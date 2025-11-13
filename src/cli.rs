use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "noet")]
#[command(about = "A CLI tool for Note blog service", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a noet working directory
    Init {
        /// Directory path (default: current directory)
        path: Option<PathBuf>,
    },

    /// Create a new article
    New {
        /// Article title
        title: Option<String>,

        /// Use a template
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Publish an article to Note
    Publish {
        /// Path to the markdown file
        file: PathBuf,

        /// Publish as draft
        #[arg(short, long)]
        draft: bool,

        /// Skip diff confirmation and force publish
        #[arg(short, long)]
        force: bool,
    },

    /// Show diff between local and remote article
    Diff {
        /// Path to the markdown file
        file: PathBuf,
    },

    /// Update an existing article
    Edit {
        /// Article ID
        id: String,

        /// Path to the markdown file
        file: PathBuf,
    },

    /// Delete an article
    Delete {
        /// Article ID
        id: String,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// List articles
    List {
        /// Username
        username: String,

        /// Page number (default: 1)
        #[arg(short, long, default_value = "1")]
        page: u32,
    },

    /// Tag management commands
    #[command(subcommand)]
    Tag(TagCommands),

    /// Magazine management commands
    #[command(subcommand)]
    Magazine(MagazineCommands),

    /// Like an article
    Like {
        /// Article key
        key: String,
    },

    /// Unlike an article
    Unlike {
        /// Article key
        key: String,
    },

    /// Show comments on an article
    Comments {
        /// Article ID
        id: String,
    },

    /// Show user information
    User {
        /// Username
        username: String,
    },

    /// Export articles to markdown files
    Export {
        /// Article key to export (if not specified, exports all articles)
        article_key: Option<String>,

        /// Export all articles from user
        #[arg(long)]
        all: bool,

        /// Username (required when using --all)
        #[arg(long)]
        username: Option<String>,

        /// Output file path (for single article) or directory (for multiple articles)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Start page for bulk export (default: 1)
        #[arg(long, default_value = "1")]
        page: u32,
    },

    /// Template management commands
    #[command(subcommand)]
    Template(TemplateCommands),

    /// Authentication commands
    #[command(subcommand)]
    Auth(AuthCommands),
}

#[derive(Subcommand)]
pub enum TagCommands {
    /// List all available tags
    List {
        /// Page number (default: 1)
        #[arg(short, long, default_value = "1")]
        page: u32,
    },

    /// Suggest tags by keyword
    Suggest {
        /// Keyword to search
        keyword: String,
    },
}

#[derive(Subcommand)]
pub enum MagazineCommands {
    /// Add article to magazine
    Add {
        /// Magazine key
        magazine: String,

        /// Article ID
        #[arg(long)]
        note_id: String,

        /// Article key
        #[arg(long)]
        note_key: String,
    },

    /// Remove article from magazine
    Remove {
        /// Magazine key
        magazine: String,

        /// Article key
        note_key: String,
    },
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

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Show authentication status and setup instructions
    Status,
}
