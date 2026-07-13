use anyhow::{Result, anyhow};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{path::PathBuf, sync::Arc};
use zstd::{bulk::decompress, decode_all};
pub mod api;

use crate::{
    configuration::MainConfig,
    knot::{
        adapters::KnotAdapter, credentials::KnotCredentials, file::KnotFile,
        resources::KnotResourcers,
    },
};

pub struct SSHAdapter {}
#[async_trait]
impl KnotAdapter for SSHAdapter {
    fn name(&self) -> String {
        String::from("SSH Adapter")
    }

    async fn get_folder(
        &self,
        folder: PathBuf,
        resources: Arc<KnotResourcers>,
        config: Arc<MainConfig>,
    ) -> Result<Vec<KnotFile>> {
        let pool = resources.ssh.as_ref().ok_or(anyhow!(
            "Resources for SSH communication weren't set properly"
        ))?;
        let session = pool.try_get_session(3).await?;
        let mut prompt = format!("./.local/bin/knot crawl --compress");
        let performance = &config.performance;
        let features = &config.features;
        if features.caching {
            prompt.push_str(" --caching");
        }
        if features.gitignore {
            prompt.push_str(" --gitignore");
        }
        let ignorer = &config.global.ignorer;
        if ignorer.len() > 0 {
            // prompt.push_str(&format!(" --ignore_patterns {ignorer:?}"));
        }
        if performance.allow_size_limit {
            prompt.push_str(&format!(" --size {}", performance.size_limit));
        }
        prompt.push_str(&format!(" -p {folder:?}"));
        println!("prompt: {prompt}");
        let (_, data) = session.call(&prompt).await?;
        let data = data.ok_or(anyhow!("Not found any data"))?.to_vec();
        let encoded_data = String::from_utf8(data)?;

        let data = STANDARD.decode(encoded_data.trim())?;
        println!("after decode the data {data:?}");

        let decompressed_data = decode_all(&data[..])?;
        let files: Vec<KnotFile> = postcard::from_bytes(&decompressed_data)?;
        Ok(files)
    }

    async fn resources(
        &self,
        credentials: &Option<KnotCredentials>,
        _config: Arc<MainConfig>,
    ) -> Result<KnotResourcers> {
        let credentials = credentials
            .as_ref()
            .ok_or(anyhow!("Credentials for SSH connection were not set"))?;
        KnotResourcers::new().ssh(&credentials, 1).await
    }
}
