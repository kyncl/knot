use std::path::PathBuf;

use anyhow::Result;

use crate::{
    KNOTS_CONFIGURATION,
    cli::{
        modification::knot_config::behavior::prompt_behaviors,
        subcommands::{add::AddSubcommand, init::prompt_knot_config},
    },
    configuration::loader::remote::{RemoteKnotConfig, RemoteKnotLoader},
    modes::setup::resolve_config_paths,
};

pub fn add_new(actions: AddSubcommand, config_path: Option<PathBuf>) -> Result<()> {
    let (_config_path, config_folder) = resolve_config_paths(config_path)?;
    let mut loaded_knots = RemoteKnotLoader::load(config_folder.join(KNOTS_CONFIGURATION))?;
    match actions {
        AddSubcommand::Remote => {
            let mut remote_knots = vec![];
            let mut first_time = true;
            while first_time
                || inquire::Confirm::new("Do you want to create a new remote knot?")
                    .with_default(false)
                    .prompt()?
            {
                if remote_knots.is_empty() {
                    eprintln!("=== Remote knots ===");
                } else {
                    eprintln!("=== New knot ===");
                }

                let config = prompt_knot_config("path/to/remote/directory", false)?;
                let behavior = prompt_behaviors()?;

                remote_knots.push(RemoteKnotConfig { config, behavior });
                first_time = false;
            }
            eprintln!("Adding new remote knots");
            let options = vec!["Append", "Rewrite"];
            let choice = inquire::Select::new(
                "It seems there are another remote configurations. What should Knot do?",
                options,
            );
            if !loaded_knots.knots.is_empty() && choice.prompt()? == "Append" {
                loaded_knots.knots.append(&mut remote_knots);
            } else {
                loaded_knots.knots = remote_knots;
            }
            loaded_knots.save(config_folder.join(KNOTS_CONFIGURATION))?;
        }
    }
    Ok(())
}
