use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use inquire::Select;
use std::{env, path::Path};

use crate::{cli::KnotArgs, utils::normalize_property};

pub fn generate_shell_complete(shell: Option<Shell>) -> Result<()> {
    let shell = if let Some(shell) = shell {
        shell
    } else {
        match detect_shell_from_env() {
            Some(s) => s,
            None => {
                let options = vec![
                    Shell::Fish,
                    Shell::Zsh,
                    Shell::Bash,
                    Shell::Elvish,
                    Shell::PowerShell,
                ];
                Select::new("Select shell you want to generate for:", options).prompt()?
            }
        }
    };
    let mut cmd = KnotArgs::command();
    let bin_name = cmd.get_name().to_lowercase();
    let mut stdout = std::io::stdout();
    generate(shell, &mut cmd, bin_name, &mut stdout);
    Ok(())
}

fn detect_shell_from_env() -> Option<Shell> {
    let shell_path = env::var("SHELL").ok()?;
    let shell_name = Path::new(&shell_path).file_name()?.to_str()?;
    match normalize_property(shell_name).as_str() {
        "fish" => Some(Shell::Fish),
        "zsh" => Some(Shell::Zsh),
        "bash" => Some(Shell::Bash),
        "pwsh" | "powershell" => Some(Shell::PowerShell),
        "elvish" => Some(Shell::Elvish),
        _ => None,
    }
}
