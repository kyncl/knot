use crate::{
    APP_FOLDER, CONFIG_FILE,
    configuration::{
        MainConfig,
        loader::{ConfigurationLoader, RemoteKnotLoader},
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
    let (config_path, app_folder) = resolve_app_paths(config_path)?;
    let loaded_conf = ConfigurationLoader::load(config_path)?;
    let patterns = loaded_conf.load_ignore_patterns(app_folder.join("ignore"))?;
    let source = Knot::from(loaded_conf.source).await?;
    let main_config = Arc::new(loaded_conf.config.ignorer(&source.path, &patterns)?);

    let mut knots = KnotManager::new(source);
    let loaded_knots = RemoteKnotLoader::load(app_folder.join("knots"))?;
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

/// Resolves the exact configuration file path and the app workspace directory.
/// Returns: (config_file_path, app_workspace_dir)
fn resolve_app_paths<P: AsRef<Path>>(input: Option<P>) -> Result<(PathBuf, PathBuf)> {
    let raw = input
        .map(|p| p.as_ref().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    if raw.is_file() {
        // Case 1: User passed an exact file (e.g., `./custom_name.toml` or `./.knot/prod.toml`)
        // The file itself is the config. The app folder is its parent directory.
        let parent = raw.parent().unwrap_or(Path::new(".")).to_path_buf();
        if parent.ends_with(APP_FOLDER) {
            Ok((raw, parent))
        } else {
            let p_with_app = parent.join(APP_FOLDER);
            // This way is valid having the config.toml inside the workspace and not inside .knot
            // Not sure if it's ok, because it can create extra configuration
            // file in sea of configs (tailwind.conf, ts.conf, ...) I guess it's users fault of not
            // making clear structure of project ¯\_(ツ)_/¯
            if p_with_app.exists() {
                Ok((raw, p_with_app))
            } else {
                Err(anyhow!(
                    "Couldn't find {APP_FOLDER}. Please create {APP_FOLDER}/ in your work space directory",
                ))
            }
        }
    } else if raw.ends_with(APP_FOLDER) {
        // Case 2: User passed the app folder directly (e.g., `./project/.knot`)
        Ok((raw.join(CONFIG_FILE), raw))
    } else {
        // Case 3: User passed a root project folder or nothing (e.g., `./project`)
        let app_dir = raw.join(APP_FOLDER);
        Ok((app_dir.join(CONFIG_FILE), app_dir))
    }
}
