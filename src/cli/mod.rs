use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use strum::Display;
pub mod autocomplete;
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
    /// Will do the main thing
    Sync,

    /// Crawling though an path and returning structure of this dir
    Crawl {
        /// Crawl through this directory
        #[arg(short = 'p', long, default_value = "./")]
        crawl_path: PathBuf,

        #[arg(long)]
        compress: bool,

        #[arg(long, default_value = "binary")]
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
    /// Directly perform file operations utilizing the Knot local adapter
    File {
        #[command(subcommand)]
        cmd: FileSubcommand,
    },
}

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub enum StructFormat {
    /// JavaScript Object Notation
    Json,
    /// Binary format
    Binary,
}

#[derive(Debug, Subcommand, PartialEq, Clone)]
pub enum FileSubcommand {
    /// Write raw bytes to a file, optionally at a specific offset (without wiping it)
    Write {
        /// The file path to write to
        path: PathBuf,
        /// Data to write, encoded as a Base64 string
        #[arg(long, value_name = "BASE64")]
        data: String,
        /// Seek offset to begin writing at
        #[arg(short, long, default_value_t = 0)]
        offset: u64,
    },
    /// Write raw bytes directly from stdin to a file
    WriteStream {
        /// Into this file knot will upload the data and than transfer it into path
        #[arg(long)]
        temporal_path: Option<PathBuf>,
        /// The file path to write to
        path: PathBuf,
    },
    ReadStream {
        /// The file path to read
        path: PathBuf,
    },
    /// Empty a file and write the specified bytes to it
    EmptyWrite {
        /// The file path to overwrite
        path: PathBuf,
        /// Data to write, encoded as a Base64 string
        #[arg(long, value_name = "BASE64")]
        data: String,
    },

    /// Read a specific range/interval of bytes from a file
    ReadInterval {
        /// The file path to read
        path: PathBuf,
        /// The starting byte position (inclusive)
        #[arg(short, long)]
        start: u64,
        /// The ending byte position (exclusive)
        #[arg(short, long)]
        end: u64,
    },

    /// Read the entire contents of a file (dangerous for large files)
    ReadFull {
        /// The file path to read
        path: PathBuf,
    },

    /// Truncate an existing file to 0 bytes, or create it empty
    Empty {
        /// The file path to empty
        path: PathBuf,
    },

    /// Move or rename a file to a new path
    Rename {
        /// Existing file path
        old_path: PathBuf,
        /// Target destination path
        new_path: PathBuf,
    },

    /// Delete a file permanently
    Delete {
        /// File path to delete
        path: PathBuf,
    },

    /// Create an empty file
    Create {
        /// File path to create
        path: PathBuf,
    },

    /// Create an directory
    CreateDir {
        /// Directory path to create
        path: PathBuf,
    },

    /// Create MULTIPLE directories
    CreateDirs {
        /// Directory path to create
        #[arg(long = "path")]
        path: Vec<PathBuf>,
    },
}
