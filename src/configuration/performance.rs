use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Serialize, Deserialize, Debug)]
#[serde(from = "PerformanceConfigDto")]
pub struct PerformanceConfig {
    /// Limiter is used well
    /// for limiting number of current task.
    /// Is made from task_limit
    #[serde(skip)]
    pub limiter: Arc<Semaphore>,
    // Right now I think it's not good idea to give the power to change this value
    #[serde(skip)]
    pub task_limit: u64,
    #[serde(with = "human_size")]
    pub size_limit: u64,
    pub allow_size_limit: bool,
}
mod human_size {
    use indicatif::HumanBytes;
    use parse_size::parse_size;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let human_str = HumanBytes(*value).to_string();
        serializer.serialize_str(&human_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_size(&s).map_err(serde::de::Error::custom)
    }
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
    /// Task limit is 1 000 concurrent tasks
    fn default() -> Self {
        let size_limit = 15 * 1024_u64.pow(3);
        let task_limit = 1_000;
        Self {
            size_limit,
            task_limit,
            limiter: Arc::new(Semaphore::new(task_limit as usize)),
            allow_size_limit: false,
        }
    }
}

#[derive(Deserialize)]
struct PerformanceConfigDto {
    // When it will make sense to modify this number
    // pub task_limit: u64,
    #[serde(with = "human_size")]
    pub size_limit: u64,
    pub allow_size_limit: bool,
}
impl From<PerformanceConfigDto> for PerformanceConfig {
    fn from(dto: PerformanceConfigDto) -> Self {
        // Self::new(dto.size_limit, dto.task_limit, dto.allow_size_limit)
        Self::new(dto.size_limit, 1000, dto.allow_size_limit)
    }
}
