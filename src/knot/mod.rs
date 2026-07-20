use crate::{
    configuration::MainConfig,
    knot::{
        KnotType::{Local, SFTP, SSH},
        adapters::{KnotAdapter, local::LocalAdapter, ssh::SSHAdapter},
        credentials::KnotCredentials,
        file::KnotFile,
        manager::RemoteKnot,
        resources::KnotResourcers,
    },
    modes::sync::sync,
    utils::paths::convert_home_path,
};
use anyhow::Result;
use std::{
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};

pub mod adapters;
pub mod credentials;
pub mod file;
pub mod file_diffs;
pub mod manager;
pub mod resources;

#[derive(Debug, PartialEq, Eq)]
pub enum KnotType {
    Local,
    SSH,
    SFTP,
}

pub struct Knot {
    adapter: Box<dyn KnotAdapter>,
    pub credentials: Option<KnotCredentials>,
    pub resources: Arc<KnotResourcers>,
    /// To path specific dir
    pub path: PathBuf,
    pub files: Vec<KnotFile>,
}
impl Knot {
    /// Creates new Knot
    /// Fails only while setting resources (creating local knot cannot return err)
    pub async fn new<P>(
        ktype: &KnotType,
        path: P,
        credentials: Option<KnotCredentials>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        // If user has remote connection to server that is running MacOS and will use '~'
        // it will, instead of /Users/{cred.username}, be /home/{cred.username}
        // My response: 'Please write absolute path to rule out any potential misinterpretation.
        // Only possible one is '~' for Linux systems'
        let username = if let Some(cred) = credentials.as_ref()
            && *ktype != KnotType::Local
        {
            Some(cred.username.clone())
        } else {
            None
        };
        let path = PathBuf::from(convert_home_path(path.as_ref(), username)?);
        let adapter: Box<dyn KnotAdapter> = {
            match ktype {
                Local => Box::new(LocalAdapter {}),
                SSH => Box::new(SSHAdapter::new()),
                SFTP => {
                    todo!("Right now there isn't SFTP adapter")
                }
            }
        };
        let resources = Arc::new(adapter.resources(&credentials).await?);
        Ok(Self {
            adapter,
            credentials,
            resources,
            path,
            files: vec![],
        })
    }

    pub fn adapter_name(&self) -> String {
        self.adapter.name()
    }

    pub fn knot_type(&self) -> KnotType {
        self.adapter.knot_type()
    }

    /// Crawl through directory self.path
    /// and return files of this folder structure
    pub async fn crawl_dir(&self, main_config: Arc<MainConfig>) -> Result<Vec<KnotFile>> {
        self.adapter
            .crawl_dir(&self.path, Arc::clone(&self.resources), main_config)
            .await
    }

    /// Crawls directory self.path and updates self.files
    pub async fn set_folder(&mut self, main_config: Arc<MainConfig>) -> Result<()> {
        let folder = self
            .adapter
            .crawl_dir(&self.path, Arc::clone(&self.resources), main_config)
            .await?;
        self.files = folder;
        Ok(())
    }

    /// Streams/writes local file content to a foreign knot target
    pub async fn transfer_to<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        foreign_knot: &Knot,
        path: P,
        foreign_path: Q,
    ) -> Result<()> {
        self.adapter
            .transfer_to(
                Arc::clone(&self.resources),
                foreign_knot,
                path.as_ref(),
                foreign_path.as_ref(),
            )
            .await
    }

    /// Truncates file to 0 bytes
    pub async fn truncate<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.adapter
            .truncate(Arc::clone(&self.resources), path.as_ref())
            .await
    }

    /// Writes data at a specific byte offset
    pub async fn write_at<P: AsRef<Path>>(&self, path: P, data: &[u8], offset: u64) -> Result<()> {
        self.adapter
            .write_at(Arc::clone(&self.resources), path.as_ref(), data, offset)
            .await
    }

    /// Renames or moves a file or directory
    pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        old_path: P,
        new_path: Q,
    ) -> Result<()> {
        self.adapter
            .rename(
                Arc::clone(&self.resources),
                old_path.as_ref(),
                new_path.as_ref(),
            )
            .await
    }

    /// Deletes a file or directory
    pub async fn delete<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.adapter
            .delete(Arc::clone(&self.resources), path.as_ref())
            .await
    }

    /// Creates an empty file
    pub async fn create<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.adapter
            .create(Arc::clone(&self.resources), path.as_ref())
            .await
    }

    /// Creates a directory
    pub async fn mkdir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.adapter
            .mkdir(Arc::clone(&self.resources), path.as_ref())
            .await
    }

    /// Creates multiple directories in batch
    pub async fn mkdir_batch<P: AsRef<Path>>(
        &self,
        paths: impl IntoIterator<Item = P>,
    ) -> Result<()> {
        let path_bufs: Vec<PathBuf> = paths
            .into_iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();
        self.adapter
            .mkdir_batch(Arc::clone(&self.resources), path_bufs)
            .await
    }

    /// Truncates the file and writes byte contents in one step
    pub async fn overwrite<P: AsRef<Path>>(&self, path: P, bytes: &[u8]) -> Result<()> {
        self.adapter
            .overwrite(Arc::clone(&self.resources), path.as_ref(), bytes)
            .await
    }

    /// Reads bytes within a byte offset range
    pub async fn read_range<P: AsRef<Path>>(&self, path: P, range: Range<u64>) -> Result<Vec<u8>> {
        self.adapter
            .read_range(Arc::clone(&self.resources), path.as_ref(), range)
            .await
    }

    /// Reads the entire file content into memory
    pub async fn read_all<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        self.adapter
            .read_all(Arc::clone(&self.resources), path.as_ref())
            .await
    }

    pub async fn sync(&self, foreign: &RemoteKnot) -> Result<()> {
        sync(self, foreign).await
    }
}
