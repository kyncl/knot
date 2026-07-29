use crate::{
    configuration::MainConfig,
    knot::{
        Knot, KnotType, credentials::KnotCredentials, file::KnotFile, resources::KnotResourcers,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use std::{ops::Range, path::Path, path::PathBuf, sync::Arc};

pub mod local;
pub mod ssh;

#[async_trait]
pub trait KnotAdapter: Send + Sync {
    /// Returns name of current adapter
    fn name(&self) -> String;
    /// Returns type of current adapter
    fn knot_type(&self) -> KnotType;
    /// Connects or initializes resources needed for operations.
    async fn resources(&self, credentials: &Option<KnotCredentials>) -> Result<KnotResourcers>;

    /// Recursively crawls the directory tree starting at `folder`
    /// to discover and build the full file structure.
    async fn crawl_dir(
        &self,
        folder: &Path,
        resources: Arc<KnotResourcers>,
        config: Arc<MainConfig>,
    ) -> Result<Vec<KnotFile>>;

    /// Streams/copies local file contents to overwrite a path on a foreign knot.
    async fn transfer_to(
        &self,
        resources: Arc<KnotResourcers>,
        foreign_knot: &Knot,
        path: &Path,
        foreign_path: &Path,
    ) -> Result<()>;

    /// Truncates a file to 0 bytes without deleting it.
    async fn truncate(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<()>;

    /// Writes the provided bytes into the file starting at `offset`
    ///
    /// # Behavior
    /// * **Overwriting:** Overwrites existing bytes starting from `offset`
    /// * **Offset `0`:** Truncates/recreates the file completely before writing
    /// * **Clamping (OOB Offset):** If `offset` exceeds the file's current size,
    ///   it is clamped to the end of the file to append the data without leaving
    ///   zero-padded sparse holes
    async fn write_at(
        &self,
        resources: Arc<KnotResourcers>,
        path: &Path,
        data: &[u8],
        offset: u64,
    ) -> Result<()>;

    /// Renames or moves a file or directory
    async fn rename(
        &self,
        resources: Arc<KnotResourcers>,
        old_path: &Path,
        new_path: &Path,
    ) -> Result<()>;

    /// Deletes a file or directory
    async fn delete(&self, resources: Arc<KnotResourcers>, paths: Vec<PathBuf>) -> Result<()>;

    /// Creates a new empty file
    async fn create(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<()>;

    /// Creates a single directory
    async fn mkdir(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<()>;

    /// Creates multiple directories in batch
    async fn mkdir_batch(&self, resources: Arc<KnotResourcers>, dirs: Vec<PathBuf>) -> Result<()>;

    /// Truncates the file and writes new contents in one operation
    async fn overwrite(
        &self,
        resources: Arc<KnotResourcers>,
        path: &Path,
        bytes: &[u8],
    ) -> Result<()>;

    /// Reads bytes within a specific byte range
    async fn read_range(
        &self,
        resources: Arc<KnotResourcers>,
        path: &Path,
        range: Range<u64>,
    ) -> Result<Vec<u8>>;

    /// Reads the entire file content into memory
    ///
    /// # Warning
    /// Can consume high memory if used on large files
    async fn read_all(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<Vec<u8>>;

    async fn archive_files(
        &self,
        resources: Arc<KnotResourcers>,
        files: Vec<PathBuf>,
        dirs: Vec<PathBuf>,
    ) -> Result<()>;

    async fn recover_files(
        &self,
        resources: Arc<KnotResourcers>,
        paths: Vec<PathBuf>,
        force: bool,
    ) -> Result<()>;
}
