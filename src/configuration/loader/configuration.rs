use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use toml_edit::{DocumentMut, value};

use crate::{
    CONFIGURATION_FOLDER,
    configuration::{MainConfig, loader::CONFIG_TEMPLATE},
    knot::KnotConfig,
    utils::toml::{apply_credentials_to_item, load_toml_file},
};

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

    pub fn to_string(&self) -> Result<String> {
        let mut doc: DocumentMut = CONFIG_TEMPLATE
            .parse()
            .context("Failed to parse existing TOML document")?;
        self.apply_to_document(&mut doc);
        Ok(doc.to_string())
    }

    pub fn default_save<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::write(path.as_ref(), CONFIG_TEMPLATE)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut saved: ConfigurationLoader = load_toml_file(path)?;
        saved.config.config_path = path.to_path_buf();
        Ok(saved)
    }

    pub fn save_ignore_patterns<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = self.ignore_patterns();
        fs::write(path, data)?;
        Ok(())
    }
    pub fn ignore_patterns(&self) -> String {
        let data: String = self.config.global.ignore_patterns.join("\n");

        let app_folder = if !data.contains(CONFIGURATION_FOLDER) {
            format!(
                r#"# {CONFIGURATION_FOLDER} contains cache and data that could be evaluated as private information.
# It is not recommended to transfer the folder
{CONFIGURATION_FOLDER}*"#
            )
        } else {
            "".to_string()
        };
        let git = if !data.contains(".git") {
            format!(
                "# .git folder is quite chunky and most of the time it's not wanted to be transferred\n.git",
            )
        } else {
            "".to_string()
        };
        let data = format!("{app_folder}\n{git}\n{data}");
        data
    }

    pub fn load_ignore_patterns<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
        let data: Vec<String> = fs::read_to_string(path)?
            .lines()
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

    fn init(&self) -> Result<String> {
        let mut doc: DocumentMut = CONFIG_TEMPLATE
            .parse()
            .context("Failed to parse initial TOML template")?;
        self.apply_to_document(&mut doc);
        Ok(doc.to_string())
    }

    fn init_save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        fs::write(path, self.init()?)
            .with_context(|| format!("Failed to create config file at {path:?}"))?;
        Ok(())
    }

    fn apply_to_document(&self, doc: &mut DocumentMut) {
        // doc["config"]["performance"]["task_limit"] =
        //     value(self.config.performance.task_limit as i64);
        doc["config"]["performance"]["allow_size_limit"] =
            value(self.config.performance.allow_size_limit);
        doc["config"]["features"]["caching"] = value(self.config.features.caching);
        doc["config"]["features"]["gitignore"] = value(self.config.features.gitignore);
        doc["config"]["features"]["compress"] = value(self.config.features.compress);

        doc["source"]["type"] = value(format!("{:?}", self.source.adapter_type));
        doc["source"]["path"] = value(self.source.path.to_string_lossy().as_ref());

        if let Some(ref cred) = self.source.credentials {
            apply_credentials_to_item(cred, &mut doc["source"]);
        }
    }
}
