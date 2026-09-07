use anyhow::Result;
use inquire::{
    Confirm, MultiSelect,
    ui::{Color, RenderConfig, Styled},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    CONFIG_FILE, CONFIGURATION_FOLDER, IGNORE_PATTERNS_FILE, KNOTS_CONFIGURATION,
    cli::modification::{
        experimental, global,
        knot_config::{
            behavior::prompt_behaviors, credentials::prompt_knot_credentials, prompt_knot_type,
            prompt_path,
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
    utils::paths::convert_home_path,
};

/// When user is setting source, it will show the selection, else it will have the smart
/// credential set (type://username@hostname:port)
pub fn prompt_knot_config(default_path: &str, is_source: bool) -> Result<KnotConfig> {
    let (cred, ktype) = if is_source {
        let ktype = prompt_knot_type()?;
        prompt_knot_credentials(Some(&ktype))?
    } else {
        prompt_knot_credentials(None)?
    };
    let autocomplete = ktype == KnotType::Local;

    Ok(KnotConfig::new(
        ktype,
        prompt_path(autocomplete, false, Some(default_path), None)?,
        cred,
    ))
}

pub fn configuration() -> Result<()> {
    eprintln!("=== Main configuration ===");
    let options = vec!["Features", "Performance", "Ignore patterns", "Experimental"];

    let render_config = RenderConfig {
        highlighted_option_prefix: Styled::new("❯").with_fg(Color::LightGreen),
        ..Default::default()
    };

    let choices = inquire::MultiSelect::new(
        "Select which sections to customize (unselected will use defaults):",
        options,
    )
    .with_render_config(render_config)
    // .with_default(&[0, 1, 2])
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
        let options = vec![
            "Enable structure caching",
            "Respect .gitignore rules",
            "Enable compression",
        ];
        let choice = MultiSelect::new("Which features should be used?", options)
            .with_default(&[0, 1])
            .prompt()?;

        config = config
            .caching(choice.contains(&"Enable structure caching"))
            .gitignore(choice.contains(&"Respect .gitignore rules"))
            .compress(choice.contains(&"Enable compression"));
    }

    if choices.contains(&"Ignore patterns") {
        let patterns = global::prompt_ignore_patterns()?;
        config = config.ignorer(Path::new("."), &patterns)?;
    }

    if choices.contains(&"Experimental") {
        config = config.async_sync(experimental::prompt_asychrnous_sync()?)
    }

    eprintln!("=== Source knot ===");
    let source = prompt_knot_config("./", true)?;

    let loader = ConfigurationLoader { source, config };
    // Because it's writing into stdout the TOML files it should make it possible
    // To pipe the results into something
    // It's printing the main configuration with the Knots configuration too
    println!("{}", toml::to_string(&loader)?);
    eprintln!("Patterns: {:?}", loader.config.global.ignore_patterns);

    let mut remote_knots = vec![];
    while inquire::Confirm::new("Do you want to create a new remote knot?")
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
        eprintln!("Saved Main Configuration into {file_path:?}");
    }

    if !loader.ignore_patterns().trim().is_empty()
        && let Some(file_path) =
            resolve_save_path(&path_to_save, IGNORE_PATTERNS_FILE, "Ignore Patterns")?
    {
        loader.save_ignore_patterns(&file_path)?;
        eprintln!("Saved Ignore Patterns into {file_path:?}");
    }

    if !remote_knots_loader.knots.is_empty()
        && let Some(file_path) =
            resolve_save_path(&path_to_save, KNOTS_CONFIGURATION, "Remote Knots")?
    {
        remote_knots_loader.save(&file_path)?;
        eprintln!("Saved Remote Knots into {file_path:?}");
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
                eprintln!("Skipped saving {description}.");
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
