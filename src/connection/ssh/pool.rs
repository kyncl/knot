use anyhow::{Result, anyhow};
use deadpool::managed::{self, Manager, Metrics, RecycleError, RecycleResult};
use std::time::Duration;
use tracing::warn;

use crate::{connection::ssh::Session, knot::credentials::KnotCredentials};

pub struct SSHManager {
    credentials: KnotCredentials,
}

impl Manager for SSHManager {
    type Type = Session;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let session = Session::connect(&self.credentials).await?;
        Ok(session)
    }

    async fn recycle(
        &self,
        session: &mut Self::Type,
        metrics: &Metrics,
    ) -> RecycleResult<Self::Error> {
        // Only run the heavy `ls` check if the connection
        // has been idle for more than 30 seconds.
        if metrics.last_used() > Duration::from_secs(30) {
            let test = session.call("ls").await;
            if let Err(e) = test {
                warn!("SSH recycle test failed: {:?}", e);
                return Err(RecycleError::Message("SSH session is dead".into()));
            }
        }
        Ok(())
    }
}

pub type Pool = managed::Pool<SSHManager>;
pub type Object = managed::Object<SSHManager>;

#[derive(Clone)]
pub struct SSHPool {
    pool: Pool,
}

impl SSHPool {
    pub async fn new(credentials: &KnotCredentials, pool_size: usize) -> Result<Self> {
        let credentials = credentials.clone();
        let manager = SSHManager { credentials };

        let pool = Pool::builder(manager)
            .max_size(pool_size)
            .build()
            .map_err(|e| anyhow!("Failed to build pool: {}", e))?;

        let instance = Self { pool };
        let mut warm_up_tasks = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            let instance_clone = instance.clone();
            let task = tokio::task::spawn(async move { instance_clone.get_session().await });
            warm_up_tasks.push(task);
        }
        for task in warm_up_tasks {
            match task.await {
                Ok(Ok(_guard)) => {}
                Ok(Err(e)) => warn!("Failed to warm up an SSH session: {e}"),
                Err(e) => warn!("Warm-up task panicked: {e}"),
            }
        }

        Ok(instance)
    }

    /// When the guard is dropped the session returns to the pool
    pub async fn get_session(&self) -> Result<Object> {
        self.pool.get().await.map_err(|e| match e {
            managed::PoolError::Backend(err) => err,
            managed::PoolError::Timeout(_) => anyhow!("Pool timeout: No available connections"),
            managed::PoolError::Closed => anyhow!("Pool is closed"),
            managed::PoolError::NoRuntimeSpecified => anyhow!("No async runtime specified"),
            managed::PoolError::PostCreateHook(e) => anyhow!("Post create error: {e}"),
        })
    }

    pub async fn try_get_session(&self, attempts: usize) -> Result<Object> {
        let mut reason = anyhow!("unknown");
        for i in 0..attempts {
            match self.get_session().await {
                Ok(sess) => return Ok(sess),
                Err(err) => {
                    warn!(
                        "Couldn't get session due to {} retrying... (try id:{})",
                        err, i
                    );
                    reason = err;
                    tokio::time::sleep(Duration::from_millis(50 * (i as u64 + 1))).await;
                }
            }
        }
        Err(anyhow!(
            "Couldn't get SSH session after {} attempts. Last reason: {}",
            attempts,
            reason
        ))
    }

    pub fn close(&self) {
        self.pool.close();
    }
}
