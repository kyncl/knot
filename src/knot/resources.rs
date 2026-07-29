use std::{sync::Arc, time::Instant};

use anyhow::Result;
use tracing::debug;

use crate::{connection::ssh::pool::SSHPool, knot::credentials::KnotCredentials};

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
        let pool = SSHPool::new(credentials, pool_size).await?;
        debug!(
            "Creating SSH pool with size {pool_size} took: {:.2?}",
            now.elapsed()
        );

        let session = pool.try_get_session(3).await?;
        let now = Instant::now();
        let bin = if let Ok((code, _)) = session.call("knot -V").await
            && code == 0
        {
            "knot"
        } else {
            "./.local/bin/knot"
        };
        debug!("Checking version of app took: {:.2?}", now.elapsed());

        self.ssh_executable = Some(bin.to_string());
        self.ssh = Some(Arc::new(pool));
        Ok(self)
    }
}
