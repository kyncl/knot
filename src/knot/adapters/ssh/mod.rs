use anyhow::{Result, anyhow};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use deadpool::managed::Object;
use std::{
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::OnceCell;
use tracing::debug;
use zstd::decode_all;
pub mod api;
pub mod rewriter;

use crate::{
    configuration::MainConfig,
    connection::ssh::pool::SSHManager,
    knot::{
        Knot, KnotType, adapters::KnotAdapter, credentials::KnotCredentials, file::KnotFile,
        resources::KnotResourcers,
    },
    utils::paths::temporal_file,
};

#[derive(Clone)]
pub struct SSHAdapter {
    bin_path: OnceCell<String>,
}

impl Default for SSHAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SSHAdapter {
    pub fn new() -> Self {
        Self {
            bin_path: OnceCell::new(),
        }
    }

    async fn get_bin_path(&self, session: &Object<SSHManager>) -> Result<&str> {
        self.bin_path
            .get_or_try_init(|| async {
                if let Ok((code, _)) = session.call("knot -V").await
                    && code == 0
                {
                    Ok(String::from("knot"))
                } else {
                    Ok(String::from("./.local/bin/knot"))
                }
            })
            .await
            .map(|s| s.as_str())
    }

    async fn execute_file_command<T>(
        &self,
        resources: Arc<KnotResourcers>,
        subcommand: T,
    ) -> Result<(u32, Option<Vec<u8>>)>
    where
        T: AsRef<str>,
    {
        let pool = resources
            .ssh
            .as_ref()
            .ok_or_else(|| anyhow!("Resources for SSH communication were not initialized"))?;

        let session = pool.try_get_session(3).await?;
        let path_to_bin = if let Some(bin) = resources.ssh_executable.as_deref() {
            bin
        } else {
            self.get_bin_path(&session).await?
        };
        let command = format!("{path_to_bin} file {}", subcommand.as_ref());
        debug!("Executing remote SSH command: {command}");
        let (code, bytes) = session.call(&command).await?;
        Ok((code, bytes.map(|b| b.to_vec())))
    }
}

#[async_trait]
impl KnotAdapter for SSHAdapter {
    fn name(&self) -> String {
        String::from("SSH Adapter")
    }

    fn knot_type(&self) -> KnotType {
        KnotType::SSH
    }

    async fn crawl_dir(
        &self,
        folder: &Path,
        resources: Arc<KnotResourcers>,
        config: Arc<MainConfig>,
    ) -> Result<Vec<KnotFile>> {
        let pool = resources.ssh.as_ref().ok_or(anyhow!(
            "Resources for SSH communication weren't set properly"
        ))?;
        let session = pool.try_get_session(3).await?;
        let path_to_bin = if let Some(bin) = resources.ssh_executable.as_deref() {
            bin
        } else {
            self.get_bin_path(&session).await?
        };

        let mut command = format!("{path_to_bin} crawl --compress");
        let performance = &config.performance;
        let features = &config.features;

        if features.caching {
            command.push_str(" --caching");
        }
        if features.gitignore {
            command.push_str(" --gitignore");
        }
        for pattern in &config.global.ignore_patterns {
            let safe_pattern = pattern.replace('\'', "'\\''");
            command.push_str(&format!(" --ignore-patterns '{safe_pattern}'"));
        }
        if performance.allow_size_limit {
            command.push_str(&format!(" --size {}", performance.size_limit));
        }
        let safe_folder = folder.display().to_string().replace('\'', "'\\''");
        command.push_str(&format!(" -p '{}'", safe_folder));
        debug!("command: {command}");
        let (code, data) = session.call(&command).await?;
        let data = data
            .ok_or(anyhow!("Not found any data code: {code}"))?
            .to_vec();
        let trimmed = data.trim_ascii();
        let decoded = STANDARD.decode(trimmed)?;
        let decompressed_data = decode_all(&decoded[..])?;
        let files: Vec<KnotFile> =
            rkyv::from_bytes::<Vec<KnotFile>, rkyv::rancor::Error>(&decompressed_data)
                .map_err(|e| anyhow!("Failed to deserialize payload with rkyv: {e}"))?;
        Ok(files)
    }

    async fn resources(&self, credentials: &Option<KnotCredentials>) -> Result<KnotResourcers> {
        let credentials = credentials
            .as_ref()
            .ok_or(anyhow!("Credentials for SSH connection were not set"))?;
        KnotResourcers::new()
            .ssh(credentials, credentials.connection_limit)
            .await
    }

    async fn truncate(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<()> {
        let cmd = format!("empty '{}'", path.display());
        let (code, _) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!("Remote empty_file failed with exit code: {}", code));
        }
        Ok(())
    }

    async fn mkdir_batch(&self, resources: Arc<KnotResourcers>, dirs: Vec<PathBuf>) -> Result<()> {
        if dirs.is_empty() {
            return Ok(());
        }
        let mut targets = dirs;
        targets.sort();
        let mut optimized_dirs: Vec<PathBuf> = Vec::with_capacity(targets.len());
        for dir in targets.into_iter().rev() {
            if let Some(last_added) = optimized_dirs.last() {
                if last_added.starts_with(&dir) {
                    continue;
                }
            }
            optimized_dirs.push(dir);
        }
        for chunk in optimized_dirs.chunks(100) {
            let paths_arg = chunk
                .iter()
                .map(|p| {
                    let safe_path = p.display().to_string().replace('\'', "'\\''");
                    format!("--path '{safe_path}'")
                })
                .collect::<Vec<_>>()
                .join(" ");
            let cmd = format!("create-dirs {paths_arg}");
            let (code, _) = self.execute_file_command(resources.clone(), &cmd).await?;
            if code != 0 {
                return Err(anyhow!(
                    "Remote create_dirs failed with exit code: {}",
                    code
                ));
            }
        }
        Ok(())
    }

    async fn mkdir(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<()> {
        let cmd = format!("create-dir '{}'", path.display());
        let (code, _) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!("Remote create_dir failed with exit code: {}", code));
        }
        Ok(())
    }

    async fn write_at(
        &self,
        resources: Arc<KnotResourcers>,
        path: &Path,
        data: &[u8],
        offset: u64,
    ) -> Result<()> {
        let base64_payload = STANDARD.encode(data);
        let cmd = format!(
            "write '{}' --data '{}' --offset {}",
            path.display(),
            base64_payload,
            offset
        );
        let (code, _) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!(
                "Remote write_into_file failed with exit code: {}",
                code
            ));
        }
        Ok(())
    }

    async fn rename(
        &self,
        resources: Arc<KnotResourcers>,
        old_path: &Path,
        new_path: &Path,
    ) -> Result<()> {
        let cmd = format!("rename '{}' '{}'", old_path.display(), new_path.display());
        let (code, _) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!(
                "Remote rename_file failed with exit code: {}",
                code
            ));
        }
        Ok(())
    }

    async fn delete(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<()> {
        let cmd = format!("delete '{}'", path.display());
        let (code, _) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!(
                "Remote delete_file failed with exit code: {}",
                code
            ));
        }
        Ok(())
    }

    async fn create(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<()> {
        let cmd = format!("create '{}'", path.display());
        let (code, _) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!(
                "Remote create_file failed with exit code: {}",
                code
            ));
        }
        Ok(())
    }

    async fn overwrite(
        &self,
        resources: Arc<KnotResourcers>,
        path: &Path,
        bytes: &[u8],
    ) -> Result<()> {
        let base64_payload = STANDARD.encode(bytes);
        let cmd = format!(
            "empty-write '{}' --data '{}'",
            path.display(),
            base64_payload
        );
        let (code, _) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!(
                "Remote empty_write_file failed with exit code: {}",
                code
            ));
        }
        Ok(())
    }

    async fn read_range(
        &self,
        resources: Arc<KnotResourcers>,
        path: &Path,
        range: Range<u64>,
    ) -> Result<Vec<u8>> {
        let cmd = format!(
            "read-interval '{}' --start {} --end {}",
            path.display(),
            range.start,
            range.end
        );
        let (code, output_bytes) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!(
                "Remote read_at_interval failed with exit code: {}",
                code
            ));
        }
        let output =
            output_bytes.ok_or_else(|| anyhow!("No read data returned from remote target"))?;
        let trimmed = output.trim_ascii();
        let decoded = STANDARD.decode(trimmed)?;
        Ok(decoded)
    }

    async fn read_all(&self, resources: Arc<KnotResourcers>, path: &Path) -> Result<Vec<u8>> {
        let cmd = format!("read-full '{}'", path.display());
        let (code, output_bytes) = self.execute_file_command(resources, &cmd).await?;
        if code != 0 {
            return Err(anyhow!(
                "Remote read_file_end failed with exit code: {}",
                code
            ));
        }
        let output =
            output_bytes.ok_or_else(|| anyhow!("No read data returned from remote target"))?;
        let trimmed = output.trim_ascii();
        let decoded = STANDARD.decode(trimmed)?;
        Ok(decoded)
    }

    async fn transfer_to(
        &self,
        resources: Arc<KnotResourcers>,
        foreign_knot: &Knot,
        path: &Path,
        foreign_path: &Path,
    ) -> Result<()> {
        let temporal_file = temporal_file(foreign_path)?;
        let result = if foreign_knot.knot_type() == KnotType::Local {
            self.stream_rewrite_local(resources, path, foreign_path, &temporal_file)
                .await
        } else {
            self.classic_rewrite(foreign_knot, &temporal_file, foreign_path, path, resources)
                .await
        };

        if let Err(err) = result {
            let mut err_msg = format!(
                "Failed to rewrite foreign knot due to: {err}. Trying to delete temporal file..."
            );
            if let Err(delete_err) = foreign_knot.delete(&temporal_file).await {
                err_msg.push_str(&format!(
                    "\nClean up of temporal file failed due to: {delete_err}"
                ));
            } else {
                err_msg.push_str("\nSuccessful clean up of temporal file");
            }
            return Err(anyhow!(err_msg));
        }
        Ok(())
    }
}
