use std::fs;

use anyhow::Result;

use crate::{
    configuration::{MainConfig, loader::ConfigurationLoader},
    knot::{KnotConfig, KnotType},
    utils::paths::temporal_file,
};

#[test]
fn test_save_load_configuration() -> Result<()> {
    fs::create_dir_all("./testing")?;
    let config_path = temporal_file("./testing/config.toml")?;
    let data = ConfigurationLoader {
        config: MainConfig::new().config_path(&config_path),
        source: KnotConfig::new(KnotType::Local, "./", None),
    };
    data.save(&config_path)?;
    let loaded_data = ConfigurationLoader::load(&config_path)?;

    assert_eq!(format!("{data:?}"), format!("{loaded_data:?}"));
    let _ = fs::remove_file(config_path);
    Ok(())
}

#[test]
fn test_configuration_default_values() -> Result<()> {
    fs::create_dir_all("./testing")?;
    let config_path = temporal_file("./testing/config.toml")?;
    ConfigurationLoader::default_save(&config_path)?;
    let data = ConfigurationLoader {
        config: MainConfig::new().config_path(&config_path),
        source: KnotConfig::new(KnotType::Local, "./", None),
    };
    let loaded_data = ConfigurationLoader::load(&config_path)?;
    assert_eq!(format!("{data:?}"), format!("{loaded_data:?}"));
    let _ = fs::remove_file(config_path);
    Ok(())
}
