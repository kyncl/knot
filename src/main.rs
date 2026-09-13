use anyhow::{Result, anyhow};
use clap::Parser;
use colored::*;
use knot::{
    cli::{
        KnotArgs, ModeArgs,
        modification::{self, adding::add_new, removing::remove_actions},
        subcommands::init,
        visualization::{config::visualize_configuration, dashboard},
    },
    configuration::MainConfig,
    modes::{
        archiving::handle_archiving,
        archiving_local::handle_local_archiving,
        crawl::crawl,
        file::handle_files,
        main_sync::{handle_sync_notifications, main_sync},
        setup::setup,
    },
    utils::{notifications::send_notification, shell_complete::generate_shell_complete},
};
use parse_size::parse_size;
use std::{sync::Arc, time::Duration};
use tracing::debug;
use tracing_appender::non_blocking;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let (non_blocking_writer, _guard) = non_blocking(std::io::stdout());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(non_blocking_writer)
        .init();
    let user_args = KnotArgs::parse();

    match user_args.mode {
        ModeArgs::Sync {
            config_path,
            notifications,
            non_interactive,
        } => {
            let (main_config, mut knots) = setup(config_path)
                .await
                .map_err(|e| anyhow!("Setup failed: {e}"))?;
            let statuses = main_sync(&mut knots, main_config, None, non_interactive).await?;
            if notifications {
                handle_sync_notifications(&statuses);
            }
        }
        ModeArgs::Daemon {
            config_path,
            notifications,
            interactive,
        } => {
            let (main_config, mut knots) = setup(config_path)
                .await
                .map_err(|e| anyhow!("Setup failed: {e}"))?;
            // Makes more sense that the daemon synchronization is no TUI by default
            let statuses =
                main_sync(&mut knots, Arc::clone(&main_config), None, !interactive).await?;
            if notifications {
                handle_sync_notifications(&statuses);
            }

            let mut last_crawled = knots.source.crawl_dir(Arc::clone(&main_config)).await?;
            last_crawled.sort_unstable_by(|a, b| a.path.cmp(&b.path));
            let mut changes_detected = false;
            eprint!("{}", "Listening...".dimmed());
            loop {
                tokio::time::sleep(Duration::from_millis(1500)).await;
                let mut new_crawled = knots.source.crawl_dir(Arc::clone(&main_config)).await?;
                new_crawled.sort_unstable_by(|a, b| a.path.cmp(&b.path));

                if new_crawled != last_crawled {
                    debug!("Found changes during daemon mode");
                    changes_detected = true;
                    last_crawled = new_crawled;
                } else if changes_detected {
                    eprintln!("\n{}", "Syncing...".dimmed().cyan());
                    let statuses = main_sync(
                        &mut knots,
                        Arc::clone(&main_config),
                        Some(new_crawled),
                        !interactive,
                    )
                    .await?;
                    if notifications {
                        handle_sync_notifications(&statuses);
                    }
                    eprintln!("{}", "Success syncing!".dimmed().green());
                    changes_detected = false;
                    eprint!("{}", "Listening...".dimmed());
                }
            }
        }
        ModeArgs::Crawl {
            format,
            compress,
            crawl_path,
            size,
            caching,
            gitignore,
            ignore_patterns,
        } => {
            let (should_limit, limit) = {
                if let Some(limit) = size {
                    let limit = parse_size(&limit).map_err(|_| anyhow!(
                        "Value `{limit}` is not supported for size. Example of valid values: `15GB`, `5MiB`, `1024B`, ..."
                    ))?;
                    (true, limit)
                } else {
                    (false, 0)
                }
            };
            let patterns = ignore_patterns.unwrap_or(vec![]);
            let config = MainConfig::new()
                .caching(caching)
                .gitignore(gitignore)
                .allow_size_limit(should_limit)
                .file_size_limit(limit)
                .ignorer(&crawl_path, &patterns)?;
            crawl(format, compress, crawl_path, config).await?;
        }
        ModeArgs::Archive {
            actions,
            index,
            config_path,
            notifications,
        } => {
            let status = handle_archiving(actions, index, config_path).await;
            if notifications {
                if let Err(err) = status {
                    send_notification(
                        "Archiving failed",
                        format!("Archiving actions failed. Cause: {err}"),
                    );
                } else {
                    send_notification(
                        "Successful archiving",
                        "All archiving actions were successfully finished",
                    );
                }
            } else {
                status?;
            }
        }
        ModeArgs::Config {
            config_path,
            format,
        } => visualize_configuration(config_path, format)?,
        ModeArgs::File { cmd } => handle_files(cmd).await?,
        ModeArgs::ArchiveLocal { actions } => handle_local_archiving(actions).await?,
        ModeArgs::Init => init::configuration()?,
        ModeArgs::Modify {
            specific_property,
            config_path,
        } => modification::modify(specific_property, config_path)?,
        ModeArgs::Add {
            actions,
            config_path,
        } => add_new(actions, config_path)?,
        ModeArgs::Remove {
            actions,
            config_path,
        } => remove_actions(actions, config_path)?,
        ModeArgs::Complete { shell } => generate_shell_complete(shell)?,
        ModeArgs::Dashboard { config_path } => dashboard::show_dashborad(config_path).await?,
    };
    Ok(())
}
