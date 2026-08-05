use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;
use strum::Display;

use crate::cli::subcommands::{
    archiving::ArchiveSubcommand, file_system::FileSubcommand, modify::ModifySubcommand,
};
pub mod autocomplete;
pub mod modification;
pub mod resolvers;
pub mod subcommands;
pub mod visualization;

#[derive(Parser, Debug)]
#[command(
    name = "Knot",
    author = "Kyncl",
    version,
    about = "TUI utility for synchronizing multiple folders across all your devices."
)]
pub struct KnotArgs {
    #[command(subcommand)]
    pub mode: ModeArgs,
}

#[derive(Debug, Subcommand, PartialEq, Clone, Display)]
pub enum ModeArgs {
    /// Synchronize directory trees across configured knots
    Sync {
        /// Path to the configuration file or workspace folder
        #[arg(short, long)]
        config_path: Option<PathBuf>,

        /// Sends notification about the synchronization process
        #[arg(short, long)]
        notifications: bool,
    },

    /// Transfer and manage archives between knots
    Archive {
        #[command(subcommand)]
        actions: Option<ArchiveSubcommand>,

        /// Target index when multiple remote knots exist
        ///
        /// Prompts interactively if omitted
        #[arg(short = 'i', long)]
        index: Option<usize>,

        /// Path to the configuration file or project workspace folder
        #[arg(short, long)]
        config_path: Option<PathBuf>,

        /// Sends notification about the archiving process
        #[arg(short, long)]
        notifications: bool,
    },

    /// Continuously monitor local directory trees and automatically sync changes
    Daemon {
        /// Path to the configuration file or workspace folder
        #[arg(short, long)]
        config_path: Option<PathBuf>,

        /// Send desktop notifications whenever a sync completes or fails
        #[arg(short, long)]
        notifications: bool,
    },

    /// Initialize your configuration file with comfy CLI
    Init,

    /// Modify specific property of configuration
    Modify {
        #[command(subcommand)]
        specific_property: ModifySubcommand,

        /// Path to the configuration file or workspace folder
        #[arg(short, long)]
        config_path: Option<PathBuf>,
    },

    /// Manage local archive files
    ArchiveLocal {
        #[command(subcommand)]
        actions: ArchiveSubcommand,
    },

    /// Scan a directory and inspect its file structure
    Crawl {
        /// Target directory path to crawl
        #[arg(short = 'p', long, default_value = "./")]
        crawl_path: PathBuf,

        /// Compress the scanned structure output
        #[arg(long)]
        compress: bool,

        /// Output serialization format for the structure
        #[arg(long, default_value = "binary")]
        format: StructFormat,

        /// Skip files exceeding this size limit (e.g., "500MB", "2GB")
        #[arg(short = 's', long)]
        size: Option<String>,

        /// Respect `.gitignore` rules when scanning files
        #[arg(short = 'g', long)]
        gitignore: bool,

        /// Cache results to speed up repeated crawls
        #[arg(short = 'c', long)]
        caching: bool,

        /// Additional file or directory patterns to ignore
        #[arg(long)]
        ignore_patterns: Option<Vec<String>>,
    },

    /// Execute low-level file operations directly
    File {
        #[command(subcommand)]
        cmd: FileSubcommand,
    },

    /// Generate shell completion scripts
    #[command(long_about = r#"
Generate shell completion scripts for Knot

To enable autocompletion, you must generate the script and put it in the correct
directory for your shell. Here are the standard commands to do this:

BASH:
    mkdir -p ~/.local/share/bash-completion/completions
    knot complete bash > ~/.local/share/bash-completion/completions/knot

ZSH:
    mkdir -p ~/.zsh/completions
    knot complete zsh > ~/.zsh/completions/_knot
    # Note: You must also add the following lines to your ~/.zshrc BEFORE compinit:
    # fpath=(~/.zsh/completions $fpath)

FISH:
    mkdir -p ~/.config/fish/completions
    knot complete fish > ~/.config/fish/completions/knot.fish

ELVISH
    mkdir -p ~/.config/elvish/lib
    knot complete elvish > ~/.config/elvish/lib/knot.elv

POWERSHELL:
    knot complete powershell | Out-String | Invoke-Expression
    # Note: To make this permanent, add the command above to your PowerShell $PROFILE
"#)]
    Complete {
        #[arg(value_enum)]
        shell: Option<Shell>,
    },
}

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub enum StructFormat {
    /// JavaScript Object Notation (JSON) format
    Json,
    /// Binary format encoded in Base64
    Binary,
}
