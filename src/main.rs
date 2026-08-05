use anyhow::{Result, anyhow};
use clap::Parser;
use futures::future;
use knot::{
    cli::{KnotArgs, ModeArgs, modification, subcommands::init},
    configuration::MainConfig,
    knot::manager::KnotManager,
    modes::{
        archiving::handle_archiving, archiving_local::handle_local_archiving, crawl::crawl,
        file::handle_files, setup::setup,
    },
    utils::{notifications::send_notification, shell_complete::generate_shell_complete},
};
use parse_size::parse_size;
use std::{sync::Arc, time::Instant};
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
        } => {
            let (main_config, mut knots) = setup(config_path)
                .await
                .map_err(|e| anyhow!("Setup failed: {e}"))?;
            println!("{main_config}");
            println!("{:?}", main_config.global.ignore_patterns);
            let start_time = Instant::now();
            let source_fut = async {
                knots
                    .source
                    .set_folder(Arc::clone(&main_config))
                    .await
                    .map_err(|e| anyhow!("Source setup failed: {e}"))
            };
            let remotes_fut = async {
                KnotManager::update_remotes(&mut knots.remotes, Arc::clone(&main_config))
                    .await
                    .map_err(|e| anyhow!("Remote update failed: {e}"))
            };
            tokio::try_join!(source_fut, remotes_fut)?;
            debug!("Update took: {:0.2?}", start_time.elapsed());

            let sync_fut = knots.remotes.iter().enumerate().map(|(index, remote)| {
                let source = &knots.source;

                let config_clone = Arc::clone(&main_config);
                async move {
                    source
                        .sync(remote, config_clone)
                        .await
                        .map_err(|e| anyhow!("Sync failed on remote #{index}: {e}"))
                }
            });

            let statuses = future::join_all(sync_fut).await;
            let status_len = statuses.len();

            if notifications {
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
        ModeArgs::File { cmd } => handle_files(cmd).await?,
        ModeArgs::ArchiveLocal { actions } => handle_local_archiving(actions).await?,
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
                        &format!("Archiving actions failed. Cause: {err}"),
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
        ModeArgs::Init => init::configuration()?,
        ModeArgs::Modify {
            specific_property,
            config_path,
        } => modification::modify(specific_property, config_path)?,
        ModeArgs::Complete { shell } => generate_shell_complete(shell)?,
    };
    Ok(())
}
