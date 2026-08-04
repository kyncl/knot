use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use toml_edit::{DocumentMut, Item, Table, value};

use crate::{
    configuration::loader::REMOTE_CONFIG_HELP_INFO,
    knot::KnotConfig,
    utils::{
        behavior::Behavior,
        toml::{apply_credentials_to_item, load_toml_file},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteKnotConfig {
    pub config: KnotConfig,
    pub behavior: Behavior,
}
pub trait ProvidesRemoteConfig {
    fn get_config(&self) -> RemoteKnotConfig;
}
impl ProvidesRemoteConfig for RemoteKnotConfig {
    fn get_config(&self) -> RemoteKnotConfig {
        self.clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteKnotLoader {
    pub knots: Vec<RemoteKnotConfig>,
}
impl RemoteKnotLoader {
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = if path.exists() {
            fs::read_to_string(path)?
        } else {
            String::new()
        };

        let mut doc: DocumentMut = content
            .parse()
            .context("Failed to parse existing TOML document")?;

        self.apply_to_document(&mut doc);
        let data = if path.exists() {
            doc.to_string()
        } else {
            let mut tmp = REMOTE_CONFIG_HELP_INFO.to_string();
            tmp.push_str(&doc.to_string());
            tmp
        };
        fs::write(path, data)?;
        Ok(())
    }

    pub fn to_string(&self) -> Result<String> {
        let mut doc: DocumentMut = REMOTE_CONFIG_HELP_INFO
            .parse()
            .context("Failed to parse existing TOML document")?;
        self.apply_to_document(&mut doc);
        Ok(doc.to_string())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_toml_file(path)
    }

    fn apply_to_document(&self, doc: &mut DocumentMut) {
        let mut knots_array = toml_edit::ArrayOfTables::new();

        for knot_config in &self.knots {
            let mut knot_table = Table::new();

            let mut behavior_table = Table::new();
            behavior_table["uniques"] = value(format!("{:?}", knot_config.behavior.uniques));
            behavior_table["conflicts"] = value(format!("{:?}", knot_config.behavior.conflicts));
            knot_table["behavior"] = Item::Table(behavior_table);

            let mut config_table = Table::new();
            config_table["type"] = value(format!("{:?}", knot_config.config.adapter_type));
            config_table["path"] = value(knot_config.config.path.to_string_lossy().as_ref());

            let mut config_item = Item::Table(config_table);
            if let Some(ref cred) = knot_config.config.credentials {
                apply_credentials_to_item(cred, &mut config_item);
            }

            knot_table["config"] = config_item;
            knots_array.push(knot_table);
        }

        doc["knots"] = Item::ArrayOfTables(knots_array);
    }
}
