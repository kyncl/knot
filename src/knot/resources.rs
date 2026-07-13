use std::sync::Arc;

use anyhow::Result;

use crate::{connection::ssh::pool::SSHPool, knot::credentials::KnotCredentials};

#[derive(Default)]
pub struct KnotResourcers {
    pub ssh: Option<Arc<SSHPool>>,
}
impl KnotResourcers {
    // Preparation for builder
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn ssh(mut self, credentials: &KnotCredentials, pool_size: usize) -> Result<Self> {
        let pool = SSHPool::new(credentials, pool_size).await?;
        self.ssh = Some(Arc::new(pool));
        Ok(self)
    }
}
