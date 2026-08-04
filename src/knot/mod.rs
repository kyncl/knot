use crate::{
    configuration::MainConfig,
    knot::{
        KnotType::{Local, SFTP, SSH},
        adapters::{
            KnotAdapter,
            local::{LocalAdapter, rewriter::stream_batch_ssh::stream_batch_ssh},
            ssh::{SSHAdapter, rewriter::stream_batch_local::stream_batch_local},
        },
        credentials::{KnotCredentials, SavedAuthMethod},
        file::KnotFile,
        remote::RemoteKnot,
        resources::KnotResourcers,
    },
    modes::sync::{get_dynamic_io_limit, sync},
    utils::{normalize_property, paths::convert_home_path},
};
use anyhow::Result;
use futures::{StreamExt, TryStreamExt, stream};
use serde::{Deserialize, Serialize};
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
pub mod remote;
pub mod resources;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum KnotType {
    Local,
    SSH,
    SFTP,
}
use serde::{Deserializer, de};
impl<'de> Deserialize<'de> for KnotType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match normalize_property(&s).as_str() {
            "local" => Ok(KnotType::Local),
            "ssh" => Ok(KnotType::SSH),
            "sftp" => Ok(KnotType::SFTP),
            _ => Err(de::Error::unknown_variant(&s, &["local", "ssh", "sftp"])),
        }
    }
}
impl std::fmt::Display for KnotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local System"),
            Self::SSH => write!(f, "SSH remote"),
            Self::SFTP => write!(f, "SFTP remote"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KnotConfig {
    #[serde(rename = "type")]
    pub adapter_type: KnotType,
    pub credentials: Option<KnotCredentials>,
    pub path: PathBuf,
}
impl KnotConfig {
    pub fn new<P: AsRef<Path>>(
        ktype: KnotType,
        path: P,
        credentials: Option<KnotCredentials>,
    ) -> Self {
        let path = path.as_ref().to_path_buf();
        Self {
            path,
            credentials,
            adapter_type: ktype,
        }
    }
}

pub struct Knot {
    adapter: Box<dyn KnotAdapter>,
    pub resources: Arc<KnotResourcers>,
    pub credentials: Option<KnotCredentials>,
    /// To path specific dir
    pub path: PathBuf,
    pub files: Vec<KnotFile>,
}
impl Knot {
    /// Creates new Knot
    /// Fails only while setting resources (creating local knot cannot return err)
    pub async fn new<P>(
        ktype: KnotType,
        path: P,
        credentials: Option<KnotCredentials>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        // Making sure that every knot has it's auth set from configuration
        let mut credentials = credentials;
        if let Some(cred) = &mut credentials {
            cred.auth = SavedAuthMethod::to_runtime_auth(&cred.config_auth)?;
        }

        // If user has remote connection to server that is running MacOS and will use '~'
        // it will, instead of /Users/{cred.username}, be /home/{cred.username}
        // My response: 'Please write absolute path to rule out any potential misinterpretation.
        // Only possible one is '~' for Linux systems'
        let username = if let Some(cred) = credentials.as_ref()
            && ktype != KnotType::Local
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

    pub async fn from(config: KnotConfig) -> Result<Self> {
        Knot::new(config.adapter_type, config.path, config.credentials).await
    }
    pub fn to_config(&self) -> KnotConfig {
        KnotConfig::new(self.knot_type(), &self.path, self.credentials.clone())
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

    pub async fn archive_files(&self, files: Vec<PathBuf>, dirs: Vec<PathBuf>) -> Result<()> {
        self.adapter
            .archive_files(Arc::clone(&self.resources), files, dirs)
            .await
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
    pub async fn delete(&self, paths: Vec<PathBuf>) -> Result<()> {
        self.adapter
            .delete(Arc::clone(&self.resources), paths)
            .await
    }

    pub async fn recover_files(&self, paths: Vec<PathBuf>, force: bool) -> Result<()> {
        self.adapter
            .recover_files(Arc::clone(&self.resources), paths, force)
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

    pub async fn sync(&self, foreign: &RemoteKnot, config: Arc<MainConfig>) -> Result<()> {
        sync(self, foreign, config).await
    }

    /// TODO: This function should be separated into adapters
    /// Returns number of files, which were transferred
    pub async fn transfer_batch<P, Q, F>(
        &self,
        foreign_knot: &Knot,
        files: &[F],
        from_root: P,
        to_root: Q,
        compress: bool,
    ) -> Result<usize>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
        F: std::borrow::Borrow<KnotFile>,
    {
        let from_root = from_root.as_ref();
        let to_root = to_root.as_ref();

        if let Some(foreign_pool) = &foreign_knot.resources.ssh
            && self.knot_type() == KnotType::Local
        {
            let foreign_session = foreign_pool.try_get_session(3).await?;
            stream_batch_ssh(
                files,
                from_root,
                to_root,
                foreign_knot,
                foreign_session,
                compress,
            )
            .await
        } else if let Some(self_pool) = &self.resources.ssh
            && foreign_knot.knot_type() == KnotType::Local
        {
            let self_session = self_pool.try_get_session(3).await?;
            stream_batch_local(
                files,
                from_root,
                to_root,
                foreign_knot,
                self_session,
                compress,
            )
            .await
        } else {
            stream::iter(files)
                .map(|file| async move {
                    let file = file.borrow();
                    let rel = file.relative_path(from_root);
                    let dest = to_root.join(rel);
                    self.transfer_to(foreign_knot, &file.path, &dest).await
                })
                .buffer_unordered(get_dynamic_io_limit(self, foreign_knot))
                .try_collect::<Vec<()>>()
                .await?;
            Ok(files.len())
        }
    }
}
