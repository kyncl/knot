use crate::knot::{adapters::ssh::SSHAdapter, resources::KnotResourcers};
use anyhow::{Result, anyhow};
use std::{path::Path, sync::Arc};
use tokio::io::AsyncWriteExt;
use tracing::debug;

impl SSHAdapter {
    pub async fn stream_rewrite_local(
        &self,
        resources: Arc<KnotResourcers>,
        path: &Path,
        foreign_path: &Path,
        temporal_file: &Path,
    ) -> Result<()> {
        let temp_file = tokio::fs::File::create(&temporal_file).await?;
        let mut buf_writer = tokio::io::BufWriter::new(temp_file);
        let pool = resources
            .ssh
            .as_ref()
            .ok_or_else(|| anyhow!("Resources for SSH communication were not set"))?;
        let session = pool.try_get_session(3).await?;
        let bin = if let Some(bin) = resources.ssh_executable.as_deref() {
            bin
        } else {
            self.get_bin_path(&session).await?
        };

        let cmd = format!("{bin} file read-stream '{}'", path.display());
        let mut channel = session.session.channel_open_session().await?;
        channel.exec(true, cmd.as_bytes()).await?;
        let mut exit_code = None;
        while let Some(msg) = channel.wait().await {
            match msg {
                russh::ChannelMsg::Data { data } => {
                    buf_writer.write_all(&data).await?;
                }
                russh::ChannelMsg::ExtendedData { data, ext: 1 } => {
                    debug!("Remote stderr: {}", String::from_utf8_lossy(&data));
                }
                russh::ChannelMsg::ExitStatus { exit_status } => {
                    exit_code = Some(exit_status);
                }
                _ => {}
            }
        }

        buf_writer.flush().await?;
        drop(buf_writer);
        if exit_code != Some(0) {
            return Err(anyhow!(
                "Remote read failed with exit code: {:?}",
                exit_code
            ));
        }
        tokio::fs::rename(temporal_file, foreign_path).await?;
        return Ok(());
    }
}
