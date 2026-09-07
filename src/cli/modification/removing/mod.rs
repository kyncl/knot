use std::path::PathBuf;

use anyhow::Result;
use inquire::{Confirm, MultiSelect};

use crate::{
    KNOTS_CONFIGURATION, cli::subcommands::remove::RemoveSubcommand,
    configuration::loader::remote::RemoteKnotLoader, modes::setup::resolve_config_paths,
};

pub fn remove_actions(actions: RemoveSubcommand, config_path: Option<PathBuf>) -> Result<()> {
    let (_config_path, config_folder) = resolve_config_paths(config_path)?;
    let mut loaded_knots = RemoteKnotLoader::load(config_folder.join(KNOTS_CONFIGURATION))?;
    match actions {
        RemoveSubcommand::Remote => {
            let options = loaded_knots
                .knots
                .iter()
                .map(|k| {
                    let cred_part = if let Some(ref cred) = k.config.credentials {
                        &format!("{}@{}:{}", cred.username, cred.host, cred.port)
                    } else {
                        "Unknown"
                    };
                    format!(
                        "{} ({cred_part}) Path: '{}'",
                        k.config.adapter_type,
                        k.config.path.display()
                    )
                })
                .collect();
            let choices =
                MultiSelect::new("Which remote Knots you want to remove", options).raw_prompt()?;
            if Confirm::new("Are you sure you want to remove these Knots?").prompt()? {
                let mut indices: Vec<usize> =
                    choices.into_iter().map(|choice| choice.index).collect();
                indices.sort_unstable_by(|a, b| b.cmp(a));
                for index in indices {
                    loaded_knots.knots.remove(index);
                }
            }
        }
    }
    loaded_knots.save(config_folder.join(KNOTS_CONFIGURATION))?;
    Ok(())
}
