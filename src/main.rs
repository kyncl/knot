use anyhow::{Result, anyhow};
use clap::Parser;
use knot::{
    cli::{KnotArgs, ModeArgs, modification, subcommands::init},
    configuration::MainConfig,
    knot::manager::KnotManager,
    modes::{
        archiving::handle_archiving, archiving_local::handle_local_archiving, crawl::crawl,
        file::handle_files, setup::setup,
    },
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
        ModeArgs::Sync { config_path } => {
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
            for (index, remote) in knots.remotes.iter().enumerate() {
                let source = &knots.source;
                source
                    .sync(remote, Arc::clone(&main_config))
                    .await
                    .map_err(|e| anyhow!("Sync failed on remote #{index}: {e}"))?;
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
        } => handle_archiving(actions, index, config_path).await?,
        ModeArgs::Init => init::configuration()?,
        ModeArgs::Modify {
            specific_property,
            config_path,
        } => modification::modify(specific_property, config_path)?,
    };
    Ok(())
}
