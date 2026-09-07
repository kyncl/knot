use anyhow::{Result, anyhow};
use clap::Parser;
use colored::*;
use futures::future;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use knot::{
    cli::{
        KnotArgs, ModeArgs,
        modification::{self, adding::add_new, removing::remove_actions},
        subcommands::init,
        visualization::{config::visualize_configuration, dashboard},
    },
    configuration::MainConfig,
    knot::{file::KnotFile, manager::KnotManager},
    modes::{
        archiving::handle_archiving, archiving_local::handle_local_archiving, crawl::crawl,
        file::handle_files, setup::setup,
    },
    utils::{
        env::is_ci_environment, notifications::send_notification,
        shell_complete::generate_shell_complete,
    },
};
use parse_size::parse_size;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
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
        ModeArgs::Dashboard {
            config_path,
            notifications,
        } => dashboard::show_dashborad(config_path, notifications).await?,
    };
    Ok(())
}

async fn main_sync(
    knots: &mut KnotManager,
    main_config: Arc<MainConfig>,
    source_files: Option<Vec<KnotFile>>,
    non_interactive: bool,
) -> Result<Vec<Result<()>>> {
    let start_time = Instant::now();
    let source_fut = async {
        if let Some(files) = source_files {
            knots.source.files = files;
            Ok(())
        } else {
            knots
                .source
                .set_folder(Arc::clone(&main_config))
                .await
                .map_err(|e| anyhow!("Source setup failed: {e}"))
        }
    };
    let remotes_fut = async {
        KnotManager::update_remotes(&mut knots.remotes, Arc::clone(&main_config))
            .await
            .map_err(|e| anyhow!("Remote update failed: {e}"))
    };
    tokio::try_join!(source_fut, remotes_fut)?;
    debug!("Update took: {:0.2?}", start_time.elapsed());

    let statuses = if main_config.experimental.async_sync {
        let sync_fut = knots.remotes.iter().enumerate().map(|(index, remote)| {
            let source = &knots.source;

            let config_clone = Arc::clone(&main_config);
            if is_ci_environment() {
                eprintln!("Doing remote Knot #{index} ...");
            }
            async move {
                source
                    .sync(remote, config_clone, non_interactive, None)
                    .await
                    .map_err(|e| anyhow!("Sync failed on remote #{index}: {e}"))
            }
        });
        future::join_all(sync_fut).await
    } else {
        let m = MultiProgress::new();
        let node_graph_pb = m.add(ProgressBar::new_spinner());
        if is_ci_environment() {
            m.set_draw_target(ProgressDrawTarget::hidden());
            node_graph_pb.set_draw_target(ProgressDrawTarget::hidden());
        }
        node_graph_pb.set_style(
            ProgressStyle::with_template(" {prefix:.cyan}{spinner:.blue}{msg}")
                .unwrap()
                // .tick_chars("󰪞󰪟󰪠󰪡󰪢󰪣󰪤󰪥")
                // .tick_chars("○◎●◎◌"),
                .tick_chars(""),
        );
        node_graph_pb.enable_steady_tick(Duration::from_millis(500));

        let total_remotes = knots.remotes.len();
        let mut statuses = Vec::with_capacity(total_remotes);

        for (index, remote) in knots.remotes.iter().enumerate() {
            let mut prefix = String::new();
            for i in 0..index {
                let node_str = "".cyan().to_string();
                let pipe_str = if i == index - 1 {
                    format!(
                        "{}{}{}",
                        "—".cyan(),
                        "—".truecolor(137, 179, 188),
                        "—".blue()
                    )
                } else {
                    "———".cyan().to_string()
                };

                prefix.push_str(&format!("{} {}", node_str, pipe_str));
            }
            node_graph_pb.set_prefix(prefix);

            let mut msg = String::new();
            for _ in (index + 1)..total_remotes {
                msg.push_str(" ———");
            }

            msg.push_str(&format!(
                "   [Syncing Knot {:02}/{:02}]",
                index + 1,
                total_remotes
            ));
            node_graph_pb.set_message(msg);

            let source = &knots.source;
            let config_clone = Arc::clone(&main_config);
            let progress = if is_ci_environment() {
                eprintln!("Doing remote Knot #{index}...");
                m.set_draw_target(ProgressDrawTarget::hidden());
                None
            } else {
                Some(&m)
            };
            statuses.push(
                source
                    .sync(remote, config_clone, non_interactive, progress)
                    .await
                    .map_err(|e| anyhow::anyhow!("Sync failed on remote #{index}: {e}")),
            );
        }

        let mut final_graph = String::new();
        for _ in 0..(total_remotes.saturating_sub(1)) {
            final_graph.push_str(&format!("{} {}", "".green(), "———".green()));
        }
        if total_remotes > 0 {
            final_graph.push_str(&format!("{}", "".green()));
        }

        node_graph_pb.set_style(ProgressStyle::with_template(" {msg:.green} ").unwrap());
        node_graph_pb.finish_with_message(format!(
            "{}    {}",
            final_graph.green(),
            "[All Knots Synced]".green()
        ));

        statuses
    };

    eprintln!(" {}", "<=> Synchronization process finished".blue());
    Ok(statuses)
}

fn handle_sync_notifications(statuses: &[Result<()>]) {
    let status_len = statuses.len();
    let mut error_happened = 0;
    let mut error_msgs = String::new();

    for status in statuses {
        if let Err(err) = status {
            error_happened += 1;
            error_msgs.push_str(&format!("{err}\n"));
        }
    }

    if error_happened == 0 {
        send_notification(
            "Successful synchronization",
            "All knots were successfully synchronized",
        );
    } else if error_happened == status_len {
        send_notification("Synchronization fully failed", error_msgs);
    } else {
        send_notification("Partial failed synchronization", error_msgs);
    }
}
