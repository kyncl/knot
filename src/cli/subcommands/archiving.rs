use clap::Subcommand;
use std::path::PathBuf;

use crate::cli::StructFormat;

#[derive(Debug, Subcommand, PartialEq, Clone)]
pub enum ArchiveSubcommand {
    /// Restore files from a target remote archive
    Recover {
        /// Root directory to inspect for recovery
        #[arg(long)]
        root_path: Option<PathBuf>,

        /// Specific archived files or paths to restore from the remote tree
        #[arg(short, long)]
        target: Vec<PathBuf>,

        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,
    },

    /// Remove stored archives
    Remove {
        /// Root directory to inspect for removal
        #[arg(long)]
        root_path: Option<PathBuf>,

        /// Specific archived files or paths to remove from the remote tree
        #[arg(short, long)]
        target: Vec<PathBuf>,

        /// Prune archives older than the specified duration (e.g., "30d", "2w")
        ///
        /// Evaluates files by modified time
        #[arg(long)]
        older_than: Option<String>,

        /// Skip interactive confirmation prompts and force deletion
        #[arg(long)]
        force: bool,
    },

    /// Launch an interactive TUI to inspect and resolve archived files
    Resolve {
        /// Root directory to inspect
        #[arg(long, default_value = "./")]
        root_path: PathBuf,
    },

    /// List all archived files
    ///
    /// For interactive TUI visualization, use the `resolve` command instead.
    List {
        /// Root directory to inspect
        #[arg(long, default_value = "./")]
        root_path: PathBuf,

        /// Compress the result
        #[arg(long)]
        compress: bool,

        /// Output serialization format for the structure
        #[arg(long, default_value = "json")]
        format: StructFormat,
    },

    /// Compress specified directories and files into archives
    Compress {
        /// Directories to include in the archive
        #[arg(short, long = "dir", value_name = "DIR")]
        dirs: Vec<PathBuf>,

        /// Individual files to include in the archive
        #[arg(short, long = "file", value_name = "FILE")]
        files: Vec<PathBuf>,
    },
}
