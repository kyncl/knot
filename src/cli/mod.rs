use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use strum::Display;
pub mod autocomplete;

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
    /// Will do the main thing
    Sync,

    /// Crawling though an path and returning structure of this dir
    Crawl {
        /// Crawl through this directory
        #[arg(short = 'p', long, default_value = "./")]
        crawl_path: PathBuf,

        #[arg(long)]
        compress: bool,

        #[arg(long, default_value = "postcard")]
        format: StructFormat,

        /// Limit files to be crawled based on their size
        /// Bigger files than this option will be skipped
        #[arg(short = 's', long)]
        size: Option<String>,

        /// Will use gitignore file to ignore files
        #[arg(short = 'g', long)]
        gitignore: bool,

        /// Allow caching
        #[arg(short = 'c', long)]
        caching: bool,

        #[arg(long)]
        ignore_patterns: Option<Vec<String>>,
    },
}

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub enum StructFormat {
    /// JavaScript Object Notation
    Json,
    /// Binary format
    Postcard,
}
