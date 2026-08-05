use std::{fs, sync::Arc, time::Instant};

use anyhow::Result;
use colored::Colorize;
use tracing::debug;

use crate::{
    cli::modification::knot_config::prompt_path,
    connection::ssh::pool::SSHPool,
    knot::{adapters::ssh::api::upload_and_prepare_server, credentials::KnotCredentials},
};

#[derive(Default)]
pub struct KnotResourcers {
    pub ssh: Option<Arc<SSHPool>>,
    /// SSH works on executing knot in remote device
    /// At start this knot needs to determined, where it's located
    /// This can take some latency issues
    pub ssh_executable: Option<String>,
}
impl KnotResourcers {
    // Preparation for builder
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn ssh(mut self, credentials: &KnotCredentials, pool_size: usize) -> Result<Self> {
        let now = Instant::now();
        let pool = Arc::new(SSHPool::new(credentials, pool_size).await?);
        debug!(
            "Creating SSH pool with size {pool_size} took: {:.2?}",
            now.elapsed()
        );

        let session = pool.try_get_session(3).await?;
        let now = Instant::now();
        let bin = if let Ok((code, _)) = session.call("knot -V").await
            && code == 0
        {
            drop(session);
            "knot"
        } else {
            if let Ok((code, _)) = session.call("./.local/bin/knot -V").await
                && code == 127
            {
                let long_part =
                    "Knot can transfer it for you. Just point to the file. Works only on";
                let spacing = " ".repeat(long_part.len() - 52);
                let path = prompt_path(
                    true,
                    true,
                    None,
                    Some(&format!(
                        "Hmm it seems your remote device doesn't have knot binary.{}]\n[{} {}",
                        spacing,
                        long_part,
                        "Unix".underline()
                    )),
                )?;
                let data = fs::read(path)?;
                drop(session);
                upload_and_prepare_server(Arc::clone(&pool), &data).await?;
            }
            "./.local/bin/knot"
        };
        debug!("Checking version of app took: {:.2?}", now.elapsed());

        self.ssh_executable = Some(bin.to_string());
        self.ssh = Some(pool);
        Ok(self)
    }
}
