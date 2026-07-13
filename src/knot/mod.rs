use crate::{
    configuration::MainConfig,
    knot::{
        KnotType::{Local, SFTP, SSH},
        adapters::{KnotAdapter, local::LocalAdapter, ssh::SSHAdapter},
        credentials::KnotCredentials,
        file::KnotFile,
        resources::KnotResourcers,
    },
};
use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub mod adapters;
pub mod credentials;
pub mod file;
pub mod file_diffs;
pub mod manager;
pub mod resources;

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
        main_config: Arc<MainConfig>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let adapter: Box<dyn KnotAdapter> = {
            match ktype {
                Local => Box::new(LocalAdapter {}),
                SSH => Box::new(SSHAdapter {}),
                SFTP => {
                    todo!("Right now there's only Local adapter")
                }
            }
        };
        let resources = Arc::new(adapter.resources(&credentials, main_config).await?);
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

    /// Crawl through directory self.path
    /// and return files of this folder
    pub async fn get_folder(&self, main_config: Arc<MainConfig>) -> Result<Vec<KnotFile>> {
        self.adapter
            .get_folder(self.path.clone(), self.resources.clone(), main_config)
            .await
    }

    /// This will call get_folder but also sets self.files
    pub async fn set_folder(&mut self, main_config: Arc<MainConfig>) -> Result<()> {
        let folder = self
            .adapter
            .get_folder(self.path.clone(), self.resources.clone(), main_config)
            .await?;
        self.files = folder;
        Ok(())
    }
}
