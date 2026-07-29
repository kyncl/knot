use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use toml_edit::{DocumentMut, value};

use crate::{APP_FOLDER, configuration::MainConfig, knot::KnotConfig, utils::behavior::Behavior};

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteKnotConfig {
    pub config: KnotConfig,
    pub behavior: Behavior,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteKnotLoader {
    pub knots: Vec<RemoteKnotConfig>,
}
impl RemoteKnotLoader {
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::write(path, toml::to_string_pretty(&self)?)?;
        Ok(())
    }
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let data =
            fs::read_to_string(path).with_context(|| format!("Failed to read file at {path:?}"))?;
        let knots =
            toml::from_str(&data).with_context(|| format!("Failed to parse TOML from {path:?}"))?;
        Ok(knots)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigurationLoader {
    pub config: MainConfig,
    pub source: KnotConfig,
}
impl ConfigurationLoader {
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            return self.init_save(path);
        }
        let content = fs::read_to_string(path)?;
        let mut doc: DocumentMut = content
            .parse()
            .context("Failed to parse existing TOML document")?;
        self.apply_to_document(&mut doc);
        fs::write(path, doc.to_string())?;
        Ok(())
    }
    /// Will save default configuration that is valid
    pub fn default_save<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::write(path.as_ref(), CONFIG_TEMPLATE)?;
        Ok(())
    }
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let config_file =
            fs::read_to_string(path).with_context(|| format!("Failed to read file at {path:?}"))?;
        let mut saved: ConfigurationLoader = toml::from_str(&config_file)
            .with_context(|| format!("Failed to parse TOML from {path:?}"))?;
        saved.config.config_path = path.to_path_buf();
        Ok(saved)
    }
    pub fn save_ignore_patterns<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data: String = self.config.global.ignore_patterns.join("\n");
        let explanation = format!(
            r#"# {APP_FOLDER} contains cache and data that could be evaluated as private information.
# It is not recommended to transfer the folder"#
        );
        let git_explanation = format!(
            "# .git folder is quite chunky and most of the time it's not wanted to be transferred"
        );
        // Ignoring by default the APP_FOLDER as an safety measure and .git, because it's usually
        // really chunky folder
        let data = format!("{explanation}\n{APP_FOLDER}/\n{git_explanation}\n.git/\n{data}");
        fs::write(path, data)?;
        Ok(())
    }
    pub fn load_ignore_patterns<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>> {
        let data: Vec<String> = fs::read_to_string(path)?
            .lines()
            .into_iter()
            .filter_map(|line| {
                if !line.starts_with("#") {
                    Some(line.to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(data)
    }
    /// Will create default init configuration
    fn init(&self) -> Result<String> {
        let mut doc: DocumentMut = CONFIG_TEMPLATE
            .parse()
            .context("Failed to parse initial TOML template")?;
        self.apply_to_document(&mut doc);
        Ok(doc.to_string())
    }
    /// Will create default init configuration and save inside the file in path
    fn init_save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        fs::write(path, self.init()?)
            .with_context(|| format!("Failed to create config file at {path:?}"))?;
        Ok(())
    }
    fn apply_to_document(&self, doc: &mut DocumentMut) {
        doc["config"]["performance"]["task_limit"] =
            value(self.config.performance.task_limit as i64);
        doc["config"]["performance"]["allow_size_limit"] =
            value(self.config.performance.allow_size_limit);
        doc["config"]["features"]["caching"] = value(self.config.features.caching);
        doc["config"]["features"]["gitignore"] = value(self.config.features.gitignore);
        doc["config"]["features"]["compress"] = value(self.config.features.compress);
        doc["source"]["type"] = value(format!("{:?}", self.source.adapter_type));
        doc["source"]["path"] = value(self.source.path.to_string_lossy().as_ref());
    }
}

const CONFIG_TEMPLATE: &str = r#"[config.performance]
# Maximum allowed concurrent tasks
task_limit = 1000

# Maximum size limit (e.g. "15.00 GB")
size_limit = "15.00 GB"
allow_size_limit = false

[config.features]
# Enable cache layer
caching = false
# Respect .gitignore files
gitignore = false
# Enable response compression
compress = false

[source]
# Adapter driver type
type = "Local"
# Path to working directory
path = "./"
"#;
