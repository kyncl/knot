use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use strum::Display;

use crate::cli::subcommands::{archiving::ArchiveSubcommand, file_system::FileSubcommand};
pub mod autocomplete;
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
    },

    /// Manage local archive files
    ArchiveLocal {
        #[command(subcommand)]
        actions: ArchiveSubcommand,
    },

    /// Transfer and manage archives between knots
    Archive {
        #[command(subcommand)]
        actions: Option<ArchiveSubcommand>,

        /// Target index when multiple remote knots exist (prompts interactively if omitted)
        #[arg(short = 'i', long)]
        index: Option<usize>,

        /// Path to the configuration file or project workspace folder
        #[arg(short, long)]
        config_path: Option<PathBuf>,
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
}

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub enum StructFormat {
    /// JavaScript Object Notation (JSON) format
    Json,
    /// Binary format encoded in Base64
    Binary,
}
