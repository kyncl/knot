use std::sync::Arc;

use serde::Serialize;
use tokio::sync::Semaphore;

#[derive(Serialize)]
pub struct PerformanceConfig {
    /// Limiter is used well
    /// for limiting number of current task.
    /// Is made from task_limit
    #[serde(skip)]
    pub limiter: Arc<Semaphore>,
    pub task_limit: u64,
    pub size_limit: u64,
    pub allow_size_limit: bool,
}
impl PerformanceConfig {
    pub fn new(size_limit: u64, task_limit: u64, allow_size_limit: bool) -> Self {
        Self {
            size_limit,
            task_limit,
            limiter: Arc::new(Semaphore::new(task_limit as usize)),
            allow_size_limit,
        }
    }
}

impl Default for PerformanceConfig {
    /// By default size limit is off (when on default limit is 15 GB)
    /// Task limit is 100 000 concurrent tasks
    fn default() -> Self {
        let size_limit = (15 * 1024_u64.pow(3)) as u64;
        let task_limit = 100_000;
        Self {
            size_limit,
            task_limit,
            limiter: Arc::new(Semaphore::new(task_limit as usize)),
            allow_size_limit: false,
        }
    }
}
