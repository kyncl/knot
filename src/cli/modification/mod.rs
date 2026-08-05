use std::path::PathBuf;

use anyhow::{Result, anyhow};
use inquire::Select;

use crate::{
    IGNORE_PATTERNS_FILE, KNOTS_CONFIGURATION,
    cli::{
        modification::{
            features::{prompt_allow_caching, prompt_allow_compression, prompt_allow_gitignore},
            global::prompt_ignore_patterns,
            knot_config::{
                behavior::{prompt_conflict_behavior, prompt_unique_behavior},
                credentials::{
                    prompt_auth, prompt_connection_limit, prompt_host, prompt_knot_credentials,
                    prompt_password, prompt_port, prompt_username,
                },
                prompt_knot_type, prompt_path,
            },
            performance::{prompt_allow_size_limit, prompt_size_limit},
        },
        resolvers::remote_indexing::resolve_remote_index,
        subcommands::modify::{KnotModifySubcommand, ModifySubcommand},
    },
    configuration::loader::{configuration::ConfigurationLoader, remote::RemoteKnotLoader},
    knot::{KnotConfig, KnotType, credentials::KnotCredentials},
    modes::setup::resolve_config_paths,
    utils::{
        behavior::Behavior,
        password::{delete_password, save_password_new},
    },
};

pub mod features;
pub mod global;
pub mod knot_config;
pub mod performance;

pub fn modify(property: ModifySubcommand, config_path: Option<PathBuf>) -> Result<()> {
    let (config_path, config_folder) = resolve_config_paths(config_path)?;
    let mut loaded_conf = ConfigurationLoader::load(&config_path)?;
    let patterns =
        ConfigurationLoader::load_ignore_patterns(config_folder.join(IGNORE_PATTERNS_FILE))
            .unwrap_or(vec![]);
    loaded_conf.config.global.ignore_patterns = patterns;

    let features = &mut loaded_conf.config.features;
    let performance = &mut loaded_conf.config.performance;
    let global = &mut loaded_conf.config.global;
    match property {
        ModifySubcommand::Caching => {
            features.caching = prompt_allow_caching()?;
        }
        ModifySubcommand::Gitignore => {
            features.gitignore = prompt_allow_gitignore()?;
        }
        ModifySubcommand::SizeLimit | ModifySubcommand::AllowSizeLimit => {
            performance.allow_size_limit = prompt_allow_size_limit()?;
            if performance.allow_size_limit {
                performance.size_limit = prompt_size_limit()?;
            }
        }
        ModifySubcommand::Compression => {
            features.compress = prompt_allow_compression()?;
        }
        ModifySubcommand::Source { properties } => {
            let knot = &mut loaded_conf.source;
            handle_knot_property(properties, knot, None)?;
        }
        ModifySubcommand::Remote { properties, index } => {
            let mut loaded_knots = RemoteKnotLoader::load(config_folder.join(KNOTS_CONFIGURATION))?;
            let knot = {
                let index = resolve_remote_index(index, &loaded_knots.knots)?;
                &mut loaded_knots
                    .knots
                    .get_mut(index)
                    .ok_or_else(|| anyhow!("Couldn't find the remote knot on index {index}"))?
            };
            handle_knot_property(properties, &mut knot.config, Some(&mut knot.behavior))?;
            loaded_knots.save(config_folder.join(KNOTS_CONFIGURATION))?;
        }
        ModifySubcommand::IgnorePatterns => {
            let mut patterns = prompt_ignore_patterns()?;
            if global.ignore_patterns.is_empty() {
                global.ignore_patterns = patterns;
            } else if !patterns.is_empty() {
                let options = vec!["Append", "Rewrite"];
                let choice = inquire::Select::new(
                    "It seems the patterns aren't empty. What should Knot do?",
                    options,
                )
                .prompt()?;
                if choice == "Append" {
                    global.ignore_patterns.append(&mut patterns);
                } else {
                    global.ignore_patterns = patterns;
                }
            }
            loaded_conf.save_ignore_patterns(config_folder.join(IGNORE_PATTERNS_FILE))?;
        }
    }

    loaded_conf.save(config_path)?;
    Ok(())
}

fn handle_knot_property(
    properties: KnotModifySubcommand,
    knot: &mut KnotConfig,
    behavior: Option<&mut Behavior>,
) -> Result<()> {
    match properties {
        KnotModifySubcommand::Type => {
            knot.adapter_type = prompt_knot_type()?;
        }
        KnotModifySubcommand::Path => {
            let autocomplete = knot.adapter_type == KnotType::Local;
            knot.path = prompt_path(autocomplete, false, Some("folder/for/remote/knot"), None)?;
        }
        KnotModifySubcommand::Port => {
            let port = prompt_port()?;
            if let Some(cred) = &mut knot.credentials {
                cred.port = port;
            } else {
                knot.credentials = Some(KnotCredentials::new().port(port));
            }
        }
        KnotModifySubcommand::Host => {
            let host = prompt_host()?;
            if let Some(cred) = &mut knot.credentials {
                cred.host = host;
            } else {
                knot.credentials = Some(KnotCredentials::new().host(host));
            }
        }
        KnotModifySubcommand::Username => {
            let username = prompt_username()?;
            if let Some(cred) = &mut knot.credentials {
                cred.username = username;
            } else {
                knot.credentials = Some(KnotCredentials::new().username(username));
            }
        }
        KnotModifySubcommand::Connections => {
            let limit = prompt_connection_limit()?;
            if let Some(cred) = &mut knot.credentials {
                cred.connection_limit = limit;
            } else {
                knot.credentials = Some(KnotCredentials::new().limit(limit));
            }
        }
        KnotModifySubcommand::Credentials => {
            let ktype = &knot.adapter_type;
            knot.credentials = prompt_knot_credentials(Some(ktype))?.0;
        }
        KnotModifySubcommand::Auth => {
            let (auth, saved_auth) = prompt_auth()?;
            if let Some(cred) = &mut knot.credentials {
                cred.auth = auth;
                cred.config_auth = saved_auth;
            } else {
                knot.credentials = Some(KnotCredentials::new());
                if let Some(cred) = &mut knot.credentials {
                    cred.auth = auth;
                    cred.config_auth = saved_auth;
                }
            }
        }
        KnotModifySubcommand::UniqueBehavior => {
            if let Some(behavior) = behavior {
                behavior.uniques = prompt_unique_behavior()?;
            } else {
                return Err(anyhow!("You cannot change on source knot behavior!"));
            }
        }
        KnotModifySubcommand::ConflictBehavior => {
            if let Some(behavior) = behavior {
                behavior.conflicts = prompt_conflict_behavior()?;
            } else {
                return Err(anyhow!("You cannot change on source knot behavior!"));
            }
        }
        KnotModifySubcommand::Password => {
            let choice =
                Select::new("What to do with password", vec!["Rewrite", "Delete"]).prompt()?;
            if let Some(cred) = &knot.credentials {
                if choice == "Rewrite" {
                    save_password_new(cred, &prompt_password()?)?;
                    println!("Password was rewritten successfully");
                } else if choice == "Delete" {
                    delete_password(cred)?;
                    println!("Password was deleted successfully");
                }
            }
        }
    }
    Ok(())
}
