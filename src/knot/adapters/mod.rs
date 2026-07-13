use std::{path::PathBuf, sync::Arc};

use crate::{
    configuration::MainConfig,
    knot::{credentials::KnotCredentials, file::KnotFile, resources::KnotResourcers},
};
use anyhow::Result;
use async_trait::async_trait;

pub mod local;
pub mod ssh;

#[async_trait]
pub trait KnotAdapter: Send + Sync {
    fn name(&self) -> String;
    async fn resources(
        &self,
        credentials: &Option<KnotCredentials>,
        config: Arc<MainConfig>,
    ) -> Result<KnotResourcers>;
    /// This will return all Files inside folder
    /// Will use path from given Knot
    async fn get_folder(
        &self,
        folder: PathBuf,
        resources: Arc<KnotResourcers>,
        config: Arc<MainConfig>,
    ) -> Result<Vec<KnotFile>>;

    // async fn rename_file(
    //     &self,
    //     config: Arc<MainConfig>,
    //     old_path: PathBuf,
    //     new_path: PathBuf,
    // ) -> Result<()>;
    // async fn delete_file(
    //     &self,
    //     config: Arc<MainConfig>,
    //     old_path: PathBuf,
    //     new_path: PathBuf,
    // ) -> Result<()>;
    // async fn create_file(&self, config: Arc<MainConfig>, path: PathBuf) -> Result<()>;
    // /// This will write some bytes (no deletion)
    // async fn write_file(&self, config: Arc<MainConfig>, path: PathBuf, bytes: &[u8]) -> Result<()>;
    // /// This will empty the file and than write some bytes
    // async fn empty_write_file(
    //     &self,
    //     config: Arc<MainConfig>,
    //     path: PathBuf,
    //     bytes: &[u8],
    // ) -> Result<()>;
    // /// Read some bytes
    // async fn read_file(&self, config: Arc<MainConfig>, path: PathBuf) -> Result<Vec<u8>>;
    // /// This will read fully the file
    // /// Can be dangerous on big files
    // async fn read_file_end(&self, config: Arc<MainConfig>, path: PathBuf) -> Result<Vec<u8>>;
}
