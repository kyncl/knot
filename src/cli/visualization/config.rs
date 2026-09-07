use colored::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};
use terminal_size::terminal_size;

use anyhow::Result;

use crate::{
    IGNORE_PATTERNS_FILE, KNOTS_CONFIGURATION,
    cli::ConfigFormat,
    configuration::loader::{configuration::ConfigurationLoader, remote::RemoteKnotLoader},
    knot::credentials::KnotCredentials,
    modes::setup::resolve_config_paths,
};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize)]
struct ConfigForPrinting {
    main: ConfigurationLoader,
    remote: RemoteKnotLoader,
    ignore_patterns: Vec<String>,
}

pub fn visualize_configuration(
    config_path: Option<PathBuf>,
    format: Option<ConfigFormat>,
) -> Result<()> {
    let term_height = terminal_size().map(|(_, h)| h.0);

    let (path, config_dir) = resolve_config_paths(config_path)?;
    let loaded_conf = ConfigurationLoader::load(path)?;
    let source = &loaded_conf.source;
    let patterns = ConfigurationLoader::load_ignore_patterns(config_dir.join(IGNORE_PATTERNS_FILE))
        .unwrap_or_default();
    let loaded_knots = RemoteKnotLoader::load(config_dir.join(KNOTS_CONFIGURATION))?;

    match format {
        Some(ConfigFormat::Toml) => {
            let config = ConfigForPrinting {
                main: loaded_conf,
                remote: loaded_knots,
                ignore_patterns: patterns,
            };
            let config = toml::to_string_pretty(&config)?;
            println!("{config}");
            return Ok(());
        }
        Some(ConfigFormat::Json) => {
            let config = ConfigForPrinting {
                main: loaded_conf,
                remote: loaded_knots,
                ignore_patterns: patterns,
            };
            let config = serde_json::to_string_pretty(&config)?;
            println!("{config}");
            return Ok(());
        }
        None => {}
    }
    let main_config = loaded_conf.config.ignorer(&source.path, &patterns)?;

    let highlight_num = |val: &dyn ToString| val.to_string().truecolor(248, 171, 129).bold();
    let highlight_behavior = |val: &dyn ToString| val.to_string().truecolor(201, 162, 249).bold();
    let mut output = String::new();
    let print_row = |output: &mut String, indent: &str, label: &str, val: &dyn Display| {
        let padded_label = format!("{:<18}", label);
        let _ = writeln!(
            output,
            "{}{:^} {}",
            indent,
            padded_label.truecolor(245, 202, 202).bold(),
            val
        );
    };

    let print_credentials = |output: &mut String, cred: &KnotCredentials, indent: &str| {
        print_row(output, indent, "Host:", &cred.host.green());
        print_row(output, indent, "Username:", &cred.username.green());
        print_row(output, indent, "Port:", &highlight_num(&cred.port));
        print_row(
            output,
            indent,
            "Connection Limit:",
            &highlight_num(&cred.connection_limit),
        );
        print_row(
            output,
            indent,
            "Auth:",
            &highlight_behavior(&cred.config_auth),
        );
    };

    let _ = writeln!(output, "\n{main_config}");
    let _ = writeln!(output, "{}", "=== SOURCE KNOT ===".cyan().bold());

    print_row(
        &mut output,
        "  ",
        "Adapter Type:",
        &highlight_behavior(&source.adapter_type),
    );
    print_row(
        &mut output,
        "  ",
        "Path:",
        &format!("{:?}", source.path).green(),
    );

    if let Some(ref cred) = source.credentials {
        let _ = writeln!(output, "  {}", "[Credentials]".yellow().bold());
        print_credentials(&mut output, cred, "    ");
    }

    let _ = writeln!(output, "\n{}", "=== REMOTE KNOTS ===".cyan().bold());
    if loaded_knots.knots.is_empty() {
        let _ = writeln!(output, "  {}", "No remote knots configured.".dimmed());
    } else {
        let multi_knot = loaded_knots.knots.len() > 1;
        for (idx, remote) in loaded_knots.knots.iter().enumerate() {
            if multi_knot {
                let _ = writeln!(output, "\n──────┐\n• {:02}: │\n──────┘", idx);
            }
            let _ = writeln!(output, "\n  {}", "[General]".yellow().bold());
            print_row(
                &mut output,
                "    ",
                "Type:",
                &highlight_behavior(&remote.config.adapter_type),
            );
            print_row(
                &mut output,
                "    ",
                "Path:",
                &format!("{:?}", remote.config.path).green(),
            );
            let _ = writeln!(output, "\n  {}", "[Behavior]".yellow().bold());
            print_row(
                &mut output,
                "    ",
                "Uniques:",
                &highlight_behavior(&remote.behavior.uniques),
            );
            print_row(
                &mut output,
                "    ",
                "Conflicts:",
                &highlight_behavior(&remote.behavior.conflicts),
            );

            if let Some(ref cred) = remote.config.credentials {
                let _ = writeln!(output, "\n  {}", "[Credentials]".yellow().bold());
                print_credentials(&mut output, cred, "    ");
            }
        }
    }

    let _ = writeln!(output, "\n{}", "=== IGNORE LIST ===".cyan().bold());
    let ignore_patterns = &main_config.global.ignore_patterns;
    if ignore_patterns.is_empty() {
        let _ = writeln!(output, "  {}", "No ignore patterns active.".dimmed());
    } else {
        for pattern in ignore_patterns {
            let _ = writeln!(output, "  • {pattern}");
        }
    }
    let _ = writeln!(output);

    render_output(&output, term_height)?;
    Ok(())
}

#[cfg(unix)]
fn render_output(output: &str, term_height: Option<u16>) -> Result<()> {
    let line_count = output.lines().count();
    let exceeds_height = term_height.is_some_and(|h| line_count > h as usize);

    if exceeds_height {
        // -R renders ANSI escape codes (colors)
        // -X prevents clearing screen on exit (optional)
        if let Ok(mut child) = Command::new("less")
            .arg("-R")
            .arg("-X")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(output.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }
    }

    print!("{output}");
    Ok(())
}

#[cfg(not(unix))]
fn render_output(output: &str, _term_height: Option<u16>) -> Result<()> {
    print!("{output}");
    Ok(())
}
