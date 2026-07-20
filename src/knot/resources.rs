use std::sync::Arc;

use anyhow::Result;

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
        let pool = SSHPool::new(credentials, pool_size).await?;
        let session = pool.try_get_session(3).await?;
        let bin = if let Ok((code, _)) = session.call("knot -V").await
            && code == 0
        {
            "knot"
        } else {
            "./.local/bin/knot"
        };
        self.ssh_executable = Some(bin.to_string());
        self.ssh = Some(Arc::new(pool));
        Ok(self)
    }
}
