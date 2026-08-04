use crate::{
    configuration::{feature::FeatureConfig, global::GlobalConfig, performance::PerformanceConfig},
    ignorer::make_git_ignore,
    utils::{paths::convert_home_path, remove_duplicates},
};
use anyhow::Result;
use colored::*;
use indicatif::HumanBytes;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Semaphore;

pub mod feature;
pub mod global;
pub mod loader;
pub mod performance;

/// Default values:
/// Ignorer: empty
/// Task_limit: 1_000
/// Size_limit: 15 GB
/// Allow_size_limit: false
/// Use caching: false
/// Use gitignore file: false
#[derive(Serialize, Deserialize, Debug)]
pub struct MainConfig {
    // Doesn't make sense to save it inside configuration file
    #[serde(skip)]
    pub config_path: PathBuf,
    // Until global has some values worthy of changing inside the file
    #[serde(skip)]
    pub global: GlobalConfig,
    pub performance: PerformanceConfig,
    pub features: FeatureConfig,
}

impl Display for MainConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = |val: bool| {
            if val {
                "ENABLED".green().bold()
            } else {
                "DISABLED".red().bold()
            }
        };

        writeln!(f, "{}", "=== Configuration Summary ===".cyan().bold())?;
        writeln!(
            f,
            "{}: {}",
            "Config File".bold(),
            self.config_path.display()
        )?;

        writeln!(f, "\n{}", "[Features]".yellow().bold())?;
        writeln!(f, "  Caching:          {}", status(self.features.caching))?;
        writeln!(f, "  Gitignore:        {}", status(self.features.gitignore))?;
        writeln!(f, "  Compress:         {}", status(self.features.compress))?;

        writeln!(f, "\n{}", "[Performance]".yellow().bold())?;
        writeln!(
            f,
            "  Task Limit:       {}",
            self.performance.task_limit.to_string().bold()
        )?;
        writeln!(
            f,
            "  Limit file size:  {}",
            status(self.performance.allow_size_limit)
        )?;
        writeln!(
            f,
            "  Size Limit:       {}",
            HumanBytes(self.performance.size_limit).to_string().bold()
        )?;

        Ok(())
    }
}

impl Default for MainConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MainConfig {
    pub fn new() -> Self {
        Self {
            config_path: PathBuf::new(),
            global: GlobalConfig::new(),
            performance: PerformanceConfig::default(),
            features: FeatureConfig::default(),
        }
    }

    pub fn config_path<P>(mut self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        self.config_path = path;
        self
    }

    /// Global settings
    /// This will take patterns and sets the global gitignore
    /// If in features is use gitignore on it will take from source knot gitignore file
    pub fn ignorer<P>(mut self, path: P, patterns: &[impl AsRef<str>]) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = {
            let path = path.as_ref();
            let path_str = convert_home_path(path, None)?;
            PathBuf::from(path_str)
        };
        self.global.ignore_patterns = remove_duplicates(patterns);
        let ignorer = make_git_ignore(&path, &self.global.ignore_patterns)?;
        self.global.ignorer = ignorer;
        Ok(self)
    }

    /// Performance settings
    /// Limit number of concurrent task that can tokio do
    pub fn task_limit(mut self, limit: u64) -> Self {
        self.performance.task_limit = limit;
        self.performance.limiter = Arc::new(Semaphore::new(limit as usize));
        self
    }

    /// Performance settings
    /// If you need ignore files bigger than limit, this is your go to
    pub fn file_size_limit(mut self, limit: u64) -> Self {
        self.performance.size_limit = limit;
        self
    }

    /// Performance settings
    pub fn allow_size_limit(mut self, allow: bool) -> Self {
        self.performance.allow_size_limit = allow;
        self
    }

    /// Feature settings
    /// If you want cache the whole structure of the folder so the next check is faster
    pub fn caching(mut self, should: bool) -> Self {
        self.features.caching = should;
        self
    }

    /// Feature settings
    /// If you want use compression on communication with remote device
    /// This may slow or fasten some operation
    pub fn compress(mut self, should: bool) -> Self {
        self.features.compress = should;
        self
    }

    /// Feature settings
    /// If you want use gitignore file from source knot
    pub fn gitignore(mut self, should: bool) -> Self {
        self.features.gitignore = should;
        self
    }
}
