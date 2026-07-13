use crate::{
    configuration::MainConfig,
    knot::{
        adapters::{KnotAdapter, local::file_crawler::local_file_crawler},
        credentials::KnotCredentials,
        file::KnotFile,
        resources::KnotResourcers,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use std::{path::PathBuf, sync::Arc};
pub mod file_crawler;
pub struct LocalAdapter {}

#[async_trait]
impl KnotAdapter for LocalAdapter {
    fn name(&self) -> String {
        String::from("Local Adapter")
    }

    async fn get_folder(
        &self,
        folder: PathBuf,
        _resources: Arc<KnotResourcers>,
        config: Arc<MainConfig>,
    ) -> Result<Vec<KnotFile>> {
        let result =
            tokio::task::spawn_blocking(move || local_file_crawler(&folder, config)).await??;
        Ok(result)
    }

    /// Local adapter doesn't need any resources
    async fn resources(
        &self,
        _credentials: &Option<KnotCredentials>,
        _config: Arc<MainConfig>,
    ) -> anyhow::Result<KnotResourcers> {
        Ok(KnotResourcers::new())
    }
}
