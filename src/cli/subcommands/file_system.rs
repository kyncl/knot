use crate::utils::compression::Compressions;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand, PartialEq, Clone)]
pub enum FileSubcommand {
    /// Write raw data bytes to a file, optionally at a specific offset without truncating
    Write {
        /// Path of the file to modify
        path: PathBuf,
        /// Data payload encoded as a Base64 string
        #[arg(long, value_name = "BASE64")]
        data: String,
        /// Byte offset position to start writing at
        #[arg(short, long, default_value_t = 0)]
        offset: u64,
    },

    /// Write data streamed from standard input (stdin) directly into a file
    WriteStream {
        /// Temporary staging location for incoming stream before moving to target path
        #[arg(long)]
        temporal_path: Option<PathBuf>,
        /// Destination file path
        path: PathBuf,
        /// Known size of file for secure transfer
        #[arg(long)]
        expected_size: Option<String>,
    },

    /// Read a tar-encoded stream from stdin and unpack it into the target root directory
    WriteBatchStream {
        /// Destination root directory where batched files will be unpacked
        #[arg(long)]
        root_path: PathBuf,

        #[arg(long, default_value_t)]
        compression: Compressions,
    },

    /// Through stdin will require file paths and through stdout will return the data of each files
    ReadBatchStream {
        /// Destination root directory where batched files will be unpacked
        #[arg(long)]
        root_path: PathBuf,

        #[arg(long, default_value_t)]
        compression: Compressions,
    },

    /// Read data from a file and stream it to standard output (stdout)
    ReadStream {
        /// Path of the file to read
        path: PathBuf,
    },

    /// Overwrite a file entirely with the provided payload
    EmptyWrite {
        /// Path of the file to overwrite
        path: PathBuf,
        /// Data payload encoded as a Base64 string
        #[arg(long, value_name = "BASE64")]
        data: String,
    },

    /// Read a specific byte range from a file
    ReadInterval {
        /// Path of the file to read
        path: PathBuf,
        /// Inclusive starting byte position
        #[arg(short, long)]
        start: u64,
        /// Exclusive ending byte position
        #[arg(short, long)]
        end: u64,
    },

    /// Read the entire contents of a file (Caution: avoid using on large files)
    ReadFull {
        /// Path of the file to read
        path: PathBuf,
    },

    /// Truncate an existing file to zero bytes, or create it if missing
    Empty {
        /// Path of the file to truncate or create
        path: PathBuf,
    },

    /// Move or rename a file or directory
    Rename {
        /// Path of the existing target
        old_path: PathBuf,
        /// New path destination
        new_path: PathBuf,
    },

    /// Permanently delete a files
    Delete {
        /// Path of the file to remove
        #[arg(long = "path")]
        path: Vec<PathBuf>,
    },

    /// Create a new empty file
    Create {
        /// Path of the file to create
        path: PathBuf,
    },

    /// Create a single directory
    CreateDir {
        /// Path of the directory to create
        path: PathBuf,
    },

    /// Create multiple directories at once
    CreateDirs {
        /// List of directory paths to create
        #[arg(long = "path")]
        path: Vec<PathBuf>,
    },
}
