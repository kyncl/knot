use crate::{
    configuration::MainConfig,
    knot::{
        Knot, KnotType,
        adapters::{
            KnotAdapter,
            local::{
                file_crawler::local_file_crawler,
                rewriter::{classic_rewrite, to_ssh::stream_rewrite_ssh},
            },
        },
        credentials::KnotCredentials,
        file::KnotFile,
        resources::KnotResourcers,
    },
    utils::paths::temporal_file,
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::{
    io::SeekFrom,
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
pub mod file_crawler;
pub mod rewriter;
pub struct LocalAdapter {}
#[async_trait]
impl KnotAdapter for LocalAdapter {
    fn name(&self) -> String {
        String::from("Local Adapter")
    }
    fn knot_type(&self) -> KnotType {
        KnotType::Local
    }

    async fn crawl_dir(
        &self,
        folder: &Path,
        _resources: Arc<KnotResourcers>,
        config: Arc<MainConfig>,
    ) -> Result<Vec<KnotFile>> {
        let folder = folder.to_path_buf();
        let result =
            tokio::task::spawn_blocking(move || local_file_crawler(&folder, config)).await??;
        Ok(result)
    }

    /// Local adapter doesn't need any resources
    async fn resources(
        &self,
        _credentials: &Option<KnotCredentials>,
    ) -> anyhow::Result<KnotResourcers> {
        Ok(KnotResourcers::new())
    }

    async fn transfer_to(
        &self,
        _resources: Arc<KnotResourcers>,
        foreign_knot: &Knot,
        path: &Path,
        foreign_path: &Path,
    ) -> Result<()> {
        let temporal_file = temporal_file(foreign_path)?;

        let result = {
            if let Some(foreign_pool) = &foreign_knot.resources.ssh {
                let foreign_session = foreign_pool.try_get_session(3).await?;
                stream_rewrite_ssh(
                    path,
                    foreign_knot,
                    foreign_session,
                    &temporal_file,
                    foreign_path,
                )
                .await
            } else if foreign_knot.knot_type() == KnotType::Local {
                tokio::fs::copy(path, &temporal_file).await?;
                foreign_knot.rename(&temporal_file, foreign_path).await
            } else {
                classic_rewrite(foreign_knot, path, &temporal_file, foreign_path).await
            }
        };

        if let Err(err) = result {
            let mut err = format!(
                "Failed to rewrite foreign knot due to: {err}. Trying to delete temporal file..."
            );
            let possible_err = foreign_knot.delete(&temporal_file).await;
            if let Err(delete_err) = possible_err {
                err.push_str(&format!(
                    "\nClean up of temporal file failed due to: {delete_err}"
                ));
            } else {
                err.push_str("\nSuccessful clean up of temporal file");
            }
            return Err(anyhow!(err));
        }

        Ok(())
    }

    async fn truncate(&self, _resources: Arc<KnotResourcers>, path: &Path) -> Result<()> {
        tokio::fs::File::create(path).await?;
        Ok(())
    }

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
        _resources: Arc<KnotResourcers>,
        path: &Path,
        data: &[u8],
        offset: u64,
    ) -> Result<()> {
        let owned_data = data.to_vec();

        let mut file = if offset == 0 {
            tokio::fs::File::create(&path).await?
        } else {
            let mut f = tokio::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .await?;

            // Clamp offset to file length to prevent zero-padding
            let file_len = f.metadata().await?.len();
            let target_offset = std::cmp::min(offset, file_len);

            f.seek(SeekFrom::Start(target_offset)).await?;
            f
        };
        file.write_all(&owned_data).await?;
        file.flush().await?;
        Ok(())
    }

    async fn rename(
        &self,
        _resources: Arc<KnotResourcers>,
        old_path: &Path,
        new_path: &Path,
    ) -> Result<()> {
        tokio::fs::rename(old_path, new_path).await?;
        Ok(())
    }

    async fn mkdir_batch(&self, _resources: Arc<KnotResourcers>, dirs: Vec<PathBuf>) -> Result<()> {
        if dirs.is_empty() {
            return Ok(());
        }
        let mut targets = dirs;
        targets.sort();
        let mut optimized_dirs: Vec<PathBuf> = Vec::with_capacity(targets.len());
        for dir in targets.into_iter().rev() {
            if let Some(last_added) = optimized_dirs.last()
                && last_added.starts_with(&dir)
            {
                continue;
            }
            optimized_dirs.push(dir);
        }
        for dir in optimized_dirs {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|e| anyhow!("Failed to create directory: {e}"))?;
        }
        Ok(())
    }

    async fn delete(&self, _resources: Arc<KnotResourcers>, path: &Path) -> Result<()> {
        if let Err(err) = tokio::fs::remove_file(&path).await {
            if err.kind() == std::io::ErrorKind::IsADirectory || err.raw_os_error() == Some(21) {
                tokio::fs::remove_dir_all(path).await?;
            } else if err.kind() != std::io::ErrorKind::NotFound {
                return Err(anyhow::Error::from(err));
            }
        }
        Ok(())
    }

    async fn create(&self, _resources: Arc<KnotResourcers>, path: &Path) -> Result<()> {
        tokio::fs::File::create(path).await?;
        Ok(())
    }

    async fn overwrite(
        &self,
        _resources: Arc<KnotResourcers>,
        path: &Path,
        bytes: &[u8],
    ) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }

    async fn read_range(
        &self,
        _resources: Arc<KnotResourcers>,
        path: &Path,
        range: Range<u64>,
    ) -> Result<Vec<u8>> {
        use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

        let mut file = tokio::fs::File::open(path).await?;
        let start = range.start;
        let end = range.end;

        if start >= end {
            return Ok(Vec::new());
        }

        let bytes_to_read = (end - start) as usize;
        file.seek(SeekFrom::Start(start)).await?;

        let mut buffer = vec![0; bytes_to_read];
        let mut total_read = 0;

        while total_read < bytes_to_read {
            match file.read(&mut buffer[total_read..]).await {
                Ok(0) => break,
                Ok(n) => total_read += n,
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(anyhow::Error::from(e)),
            }
        }

        buffer.truncate(total_read);
        Ok(buffer)
    }

    async fn mkdir(&self, _resources: Arc<KnotResourcers>, path: &Path) -> Result<()> {
        tokio::fs::create_dir_all(path).await?;
        Ok(())
    }

    /// This will read fully the file
    /// Can be dangerous on big files
    async fn read_all(&self, _resources: Arc<KnotResourcers>, path: &Path) -> Result<Vec<u8>> {
        let bytes = tokio::fs::read(path).await?;
        Ok(bytes)
    }
}
