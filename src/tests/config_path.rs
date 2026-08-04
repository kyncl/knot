use crate::{
    CONFIG_FILE, CONFIGURATION_FOLDER, modes::setup::resolve_config_paths,
    utils::paths::temporal_file,
};
use anyhow::Result;

use std::{
    fs::{self, File},
    path::PathBuf,
};

#[test]
fn resolve_config_paths_standard_dir() -> Result<()> {
    let temp_workspace = temporal_file(PathBuf::from("testing").join("resolve_config"))?;
    fs::create_dir_all(&temp_workspace)?;
    let knot_dir = temp_workspace.join(CONFIGURATION_FOLDER);
    fs::create_dir_all(&knot_dir)?;

    let result = resolve_config_paths(Some(temp_workspace.clone()))?;
    assert_eq!(result.0, knot_dir.join(CONFIG_FILE));
    assert_eq!(result.1, knot_dir);

    fs::remove_dir_all(&knot_dir)?;
    fs::remove_dir_all(temp_workspace)?;
    Ok(())
}

#[test]
fn resolve_config_paths_custom_knot_dir() -> Result<()> {
    let temp_workspace = temporal_file(PathBuf::from("testing").join("resolve_config"))?;
    fs::create_dir_all(&temp_workspace)?;
    let custom_knot_dir = temp_workspace.join(".knot_custom");
    fs::create_dir_all(&custom_knot_dir)?;

    let result = resolve_config_paths(Some(custom_knot_dir.clone()))?;
    assert_eq!(result.0, custom_knot_dir.join(CONFIG_FILE));
    assert_eq!(result.1, custom_knot_dir);

    fs::remove_dir_all(&custom_knot_dir)?;
    fs::remove_dir_all(temp_workspace)?;
    Ok(())
}

#[test]
fn resolve_config_paths_file_inside_config_dir() -> Result<()> {
    let temp_workspace = temporal_file(PathBuf::from("testing").join("resolve_config"))?;
    fs::create_dir_all(&temp_workspace)?;
    let knot_dir = temp_workspace.join(CONFIGURATION_FOLDER);

    fs::create_dir_all(&knot_dir)?;
    let prod_file = knot_dir.join("prod.toml");
    File::create(&prod_file)?;

    let result = resolve_config_paths(Some(prod_file.clone()))?;
    assert_eq!(result.0, prod_file);
    assert_eq!(result.1, knot_dir);

    fs::remove_dir_all(&knot_dir)?;
    fs::remove_dir_all(temp_workspace)?;
    Ok(())
}

#[test]
fn resolve_config_paths_file_outside_config_dir() -> Result<()> {
    let temp_workspace = temporal_file(PathBuf::from("testing").join("resolve_config"))?;
    fs::create_dir_all(&temp_workspace)?;
    let knot_dir = temp_workspace.join(CONFIGURATION_FOLDER);
    fs::create_dir_all(&knot_dir)?;
    let external_file = temp_workspace.join("prod_knot.toml");
    File::create(&external_file)?;

    let result = resolve_config_paths(Some(external_file.clone()))?;
    assert_eq!(result.0, external_file);
    assert_eq!(result.1, knot_dir);

    fs::remove_file(&external_file)?;
    fs::remove_dir_all(&knot_dir)?;
    fs::remove_dir_all(temp_workspace)?;
    Ok(())
}

#[test]
fn resolve_config_paths_missing_config_folder() -> Result<()> {
    let temp_workspace = temporal_file(PathBuf::from("testing").join("resolve_config"))?;
    fs::create_dir_all(&temp_workspace)?;
    let result = resolve_config_paths(Some(temp_workspace.clone()));
    assert!(result.is_err());
    fs::remove_dir_all(temp_workspace)?;
    Ok(())
}
