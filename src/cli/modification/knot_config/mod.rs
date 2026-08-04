use std::path::PathBuf;

use anyhow::Result;
use inquire::{
    Select, Text,
    ui::{Color, RenderConfig, Styled},
};

use crate::{cli::autocomplete::path::FilePathCompleter, knot::KnotType};

pub mod behavior;
pub mod credentials;

pub fn prompt_knot_type() -> Result<KnotType> {
    let options = vec![KnotType::Local, KnotType::SSH];

    let choice = Select::new("Select Knot Type:", options)
        .with_help_message("Determines how the target environment is accessed")
        .prompt()?;

    Ok(choice)
}

pub fn prompt_path(
    use_autocomplete: bool,
    use_files: bool,
    place_holder: Option<&str>,
    help_message: Option<&str>,
) -> Result<PathBuf> {
    let mut render_config = RenderConfig::default();
    render_config.highlighted_option_prefix = Styled::new("❯").with_fg(Color::LightGreen);
    let mut prompt = Text::new("Enter path:");
    if let Some(place_holder) = place_holder {
        prompt = prompt.with_placeholder(place_holder);
    }
    if use_autocomplete {
        prompt = prompt
            .with_autocomplete(FilePathCompleter::new(use_files))
            .with_help_message("Press <Tab> to auto-complete local file paths");
    } else {
        prompt = prompt.with_help_message("Enter absolute or relative path on the remote host");
    }
    if let Some(msg) = help_message {
        prompt = prompt.with_help_message(msg);
    }
    let path = prompt.with_render_config(render_config).prompt()?;
    let path = if let Some(place_holder) = place_holder
        && path.is_empty()
    {
        place_holder.to_string()
    } else {
        path
    };
    Ok(PathBuf::from(path))
}
