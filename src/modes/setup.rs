use crate::{
    CONFIG_FILE, CONFIGURATION_FOLDER, IGNORE_PATTERNS_FILE, KNOTS_CONFIGURATION,
    configuration::{
        MainConfig,
        loader::{configuration::ConfigurationLoader, remote::RemoteKnotLoader},
    },
    knot::{Knot, KnotType, adapters::ssh::api::upload_and_prepare_server, manager::KnotManager},
};
use anyhow::{Result, anyhow};
use inquire::{Confirm, Text};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Will read configuration from ./APP_FOLDER/CONFIG_FILE
/// Ignore patterns from ./APP_FOLDER/ignore
/// Remote knots from ./APP_FOLDER/knots
pub async fn setup<P: AsRef<Path>>(
    config_path: Option<P>,
) -> Result<(Arc<MainConfig>, KnotManager)> {
    let (config_path, config_dir) = resolve_config_paths(config_path)?;
    let loaded_conf = ConfigurationLoader::load(config_path)?;
    let patterns = ConfigurationLoader::load_ignore_patterns(config_dir.join(IGNORE_PATTERNS_FILE))
        .unwrap_or(vec![]);
    let source = Knot::from(loaded_conf.source).await?;
    let main_config = Arc::new(loaded_conf.config.ignorer(&source.path, &patterns)?);

    let mut knots = KnotManager::new(source);
    let loaded_knots = RemoteKnotLoader::load(config_dir.join(KNOTS_CONFIGURATION))?;
    for remote in loaded_knots.knots {
        let knot = Knot::from(remote.config).await?;
        let behavior = remote.behavior;
        knots.add_remote(knot, behavior);
    }

    let mut path = None;
    for remote in &knots.remotes {
        if let Some(ref pool) = remote.knot.resources.ssh
            && remote.knot.resources.ssh_executable.is_none()
        {
            let pool = Arc::clone(pool);
            if path.is_none() {
                path = Some(Text::new(&format!("{}\n{}\n{}\n{}",
                            "It seems knot is not installed on remote device. It is mandatory to have installed.",
                            "Knot can transfer the executable inside, but needs you to point at it.",
                            "Note: This operations was tested only on linux servers.",
                            "It is recommended to do this operation manually on other Operating systems.",
                )).prompt()?);
            }
            if let Some(ref path) = path {
                let connection = {
                    let path = remote.knot.path.display();
                    let ktype = remote.knot.knot_type();
                    let ktype_str = format!("{:?}", ktype).to_lowercase();
                    match ktype {
                        KnotType::Local => {
                            format!("{ktype_str}://{path}")
                        }
                        KnotType::SSH | KnotType::SFTP => {
                            let (username, host, port) =
                                if let Some(ref cred) = remote.knot.credentials {
                                    (cred.host.clone(), cred.host.clone(), cred.port)
                                } else {
                                    ("unknown".to_string(), "unknown".to_string(), 22)
                                };
                            format!("{ktype_str}://{username}@{host}:{port}")
                        }
                    }
                };
                if Confirm::new(&format!("Use {path:?} for {connection}"))
                    .with_default(true)
                    .prompt()?
                {
                    upload_and_prepare_server(pool, &fs::read(path)?).await?;
                }
            }
        }
    }

    Ok((main_config, knots))
}

/// Resolves the exact configuration file path and the configuration directory based on user input.
///
/// This function allows flexible configuration structures, enabling users to swap between
/// multiple configurations (e.g., `prod.toml`, `dev.toml`) while ensuring all setups
/// share a centralized configuration directory prefix (`.knot`).
///
/// # Resolution Rules
///
/// 1. **Configuration Directory**: The core configuration directory must start with the
///    `CONFIGURATION_FOLDER` prefix (e.g., `.knot` or `.knot_custom`).
/// 2. **File Inside Config Dir**: If a file is provided inside a valid config directory
///    (e.g., `.knot/prod.toml`), that file is used as the main config, and its parent is the config dir.
/// 3. **File Outside Config Dir**: If a file is provided in the workspace root (e.g., `prod_knot.toml`),
///    it is accepted as the main config, but the system will rely on the standard `.knot` directory
///    existing alongside it for other essential configurations.
/// 4. **Directory Input**: If a directory is provided (or nothing is passed), it resolves to the standard
///    `CONFIG_FILE` (`knot.toml`) inside the `.knot` folder of that target directory.
///
/// # Examples of Valid Paths
///
/// Assuming `CONFIGURATION_FOLDER` = `".knot"` and `CONFIG_FILE` = `"knot.toml"`:
///
/// | Input (`input`)               | Resolved `(config_file_path, config_dir)`   |
/// |-------------------------------|---------------------------------------------|
/// | `None` or `"."`               | `./.knot/knot.toml`, `./.knot`              |
/// | `./project`                   | `./project/.knot/knot.toml`, `./project/.knot` |
/// | `./.knot_custom`              | `./.knot_custom/knot.toml`, `./.knot_custom`|
/// | `./.knot/prod.toml`           | `./.knot/prod.toml`, `./.knot`              |
/// | `./.knot_custom/dev.toml`     | `./.knot_custom/dev.toml`, `./.knot_custom` |
/// | `./prod_knot.toml`            | `./prod_knot.toml`, `./.knot`               |
///
/// # Returns
/// - `Ok((config_file_path, config_dir))` on success.
/// - `Err` if the required `CONFIGURATION_FOLDER` does not exist in the targeted workspace.
pub fn resolve_config_paths<P: AsRef<Path>>(input: Option<P>) -> Result<(PathBuf, PathBuf)> {
    let raw = input
        .map(|p| p.as_ref().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    if raw.is_file() {
        let parent = raw.parent().unwrap_or(Path::new(".")).to_path_buf();
        let parent_name = parent.file_name().unwrap_or_default().to_string_lossy();

        if parent_name.starts_with(CONFIGURATION_FOLDER) {
            Ok((raw, parent))
        } else {
            let p_with_app = parent.join(CONFIGURATION_FOLDER);
            if p_with_app.exists() {
                Ok((raw, p_with_app))
            } else {
                Err(anyhow!(
                    "Couldn't find {CONFIGURATION_FOLDER}. Please create {CONFIGURATION_FOLDER}/ in your workspace directory"
                ))
            }
        }
    } else {
        let dir_name = raw.file_name().unwrap_or_default().to_string_lossy();
        if dir_name.starts_with(CONFIGURATION_FOLDER) {
            Ok((raw.join(CONFIG_FILE), raw))
        } else {
            let config_dir = raw.join(CONFIGURATION_FOLDER);
            if config_dir.exists() {
                Ok((config_dir.join(CONFIG_FILE), config_dir))
            } else {
                Err(anyhow!(
                    "Couldn't find {CONFIGURATION_FOLDER}. Please create {CONFIGURATION_FOLDER}/ in your workspace directory"
                ))
            }
        }
    }
}
