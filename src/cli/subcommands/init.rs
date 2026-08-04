use anyhow::Result;
use inquire::{
    Confirm,
    ui::{Color, RenderConfig, Styled},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    CONFIG_FILE, CONFIGURATION_FOLDER, IGNORE_PATTERNS_FILE, KNOTS_CONFIGURATION,
    cli::modification::{
        features, global,
        knot_config::{
            behavior::{prompt_conflict_behavior, prompt_unique_behavior},
            credentials::prompt_knot_credentials,
            prompt_knot_type, prompt_path,
        },
        performance,
    },
    configuration::{
        MainConfig,
        loader::{
            configuration::ConfigurationLoader,
            remote::{RemoteKnotConfig, RemoteKnotLoader},
        },
    },
    knot::{KnotConfig, KnotType},
    utils::{behavior::Behavior, paths::convert_home_path},
};

fn prompt_knot_config(default_path: &str) -> Result<KnotConfig> {
    let ktype = prompt_knot_type()?;
    let autocomplete = ktype == KnotType::Local;

    Ok(KnotConfig::new(
        ktype.clone(),
        prompt_path(autocomplete, false, Some(default_path), None)?,
        prompt_knot_credentials(Some(&ktype))?.0,
    ))
}

pub fn configuration() -> Result<()> {
    println!("=== Main configuration ===");
    let options = vec!["Features", "Performance", "Ignore patterns"];

    let mut render_config = RenderConfig::default();
    render_config.highlighted_option_prefix = Styled::new("❯").with_fg(Color::LightGreen);

    let choices = inquire::MultiSelect::new(
        "Select which sections to customize (unselected will use defaults):",
        options,
    )
    .with_render_config(render_config)
    .with_all_selected_by_default()
    .prompt()?;
    let mut config = MainConfig::new().task_limit(1000);
    if choices.contains(&"Performance") {
        let allow_size = performance::prompt_allow_size_limit()?;
        let size = if allow_size {
            performance::prompt_size_limit()?
        } else {
            0
        };
        config = config.allow_size_limit(allow_size).file_size_limit(size);
    }

    if choices.contains(&"Features") {
        config = config
            .caching(features::prompt_allow_caching()?)
            .gitignore(features::prompt_allow_gitignore()?)
            .compress(features::prompt_allow_compression()?);
    }

    if choices.contains(&"Ignore patterns") {
        let patterns = global::prompt_ignore_patterns()?;
        config = config.ignorer(Path::new("."), &patterns)?;
    }

    println!("=== Source knot ===");
    let source = prompt_knot_config("folder/for/source/knot")?;

    let loader = ConfigurationLoader { source, config };
    println!("{}", toml::to_string(&loader)?);
    println!("Patterns: {:?}", loader.config.global.ignore_patterns);

    let mut remote_knots = vec![];
    while inquire::Confirm::new("Do you want to create a new remote knot?")
        .with_default(false)
        .prompt()?
    {
        if remote_knots.is_empty() {
            println!("=== Remote knots ===");
        } else {
            println!("=== New knot ===");
        }

        let config = prompt_knot_config("folder/for/remote/knot")?;
        let behavior = Behavior {
            uniques: prompt_unique_behavior()?,
            conflicts: prompt_conflict_behavior()?,
        };

        remote_knots.push(RemoteKnotConfig { config, behavior });
    }

    let remote_knots_loader = RemoteKnotLoader {
        knots: remote_knots,
    };
    if !remote_knots_loader.knots.is_empty() {
        println!("{}", toml::to_string_pretty(&remote_knots_loader)?);
    }

    let save_it = Confirm::new(&format!(
        "Do you want to save the configuration into ./{CONFIGURATION_FOLDER}?"
    ))
    .with_default(true)
    .prompt()?;

    let mut path_to_save = if save_it {
        PathBuf::from(format!("./{CONFIGURATION_FOLDER}"))
    } else {
        prompt_path(
            true,
            false,
            None,
            Some("Folder, where all configuration files will live in"),
        )?
    };
    path_to_save = PathBuf::from(convert_home_path(&path_to_save, None)?);
    fs::create_dir_all(&path_to_save)?;

    if let Some(file_path) = resolve_save_path(&path_to_save, CONFIG_FILE, "Main Configuration")? {
        loader.save(&file_path)?;
        println!("Saved Main Configuration into {file_path:?}");
    }

    if !loader.ignore_patterns().trim().is_empty()
        && let Some(file_path) =
            resolve_save_path(&path_to_save, IGNORE_PATTERNS_FILE, "Ignore Patterns")?
        {
            loader.save_ignore_patterns(&file_path)?;
            println!("Saved Ignore Patterns into {file_path:?}");
        }

    if !remote_knots_loader.knots.is_empty()
        && let Some(file_path) =
            resolve_save_path(&path_to_save, KNOTS_CONFIGURATION, "Remote Knots")?
        {
            remote_knots_loader.save(&file_path)?;
            println!("Saved Remote Knots into {file_path:?}");
        }
    Ok(())
}

/// Resolves the file path interactively, handling overwrites and renames.
/// Returns `Ok(Some(PathBuf))` if the file should be saved, or `Ok(None)` if skipped.
fn resolve_save_path(
    dir_path: &Path,
    default_name: &str,
    description: &str,
) -> Result<Option<PathBuf>> {
    let mut file_path = dir_path.join(default_name);

    if file_path.exists() {
        let overwrite = Confirm::new(&format!(
            "Found {:?} file. Do you want to overwrite it?",
            file_path.file_name().unwrap()
        ))
        .with_default(true)
        .prompt()?;

        if !overwrite {
            let rename =
                Confirm::new("Do you want to save it under a different file name instead?")
                    .with_default(false)
                    .prompt()?;

            if rename {
                let new_name = inquire::Text::new("Enter the new file name:").prompt()?;
                file_path = dir_path.join(new_name);
            } else {
                println!("Skipped saving {description}.");
                return Ok(None); // User chose not to save
            }
        }
    }

    // Ensure the parent directory exists (especially if they typed a nested rename)
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(Some(file_path))
}
