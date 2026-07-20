use anyhow::{Result, anyhow};
use clap::Parser;
use inquire::{Password, Text};
use knot::{
    cli::{KnotArgs, ModeArgs, autocomplete::path::FilePathCompleter},
    configuration::MainConfig,
    knot::{
        Knot, KnotType, adapters::ssh::api::upload_and_prepare_server,
        credentials::KnotCredentials, manager::KnotManager,
    },
    modes::{crawl::crawl, file::handle_files},
    utils::behavior::Behavior,
};
use parse_size::parse_size;
use std::{fs, sync::Arc, time::Instant};
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
        ModeArgs::Sync => {
            let path = Text::new("Path to folder for checking")
                .with_autocomplete(FilePathCompleter::default())
                .prompt()?;
            let main_config = Arc::new(
                MainConfig::new()
                    .caching(true)
                    .gitignore(true)
                    .task_limit(10_000)
                    .file_size_limit(parse_size("15GB")?)
                    .ignorer(&path, &["node_modules/"])?,
            );
            let source = Knot::new(&KnotType::Local, &path, None).await?;
            let mut knots = KnotManager::new(source);

            let path = Text::new("Path to folder for checking in SSH")
                .with_help_message(
                    "Please write absolute path to rule out any potential misinterpretation.\nOnly possible one is '~' for Linux systems",
                )
                .prompt()?;
            let remote = Knot::new(
                &KnotType::SSH,
                &path,
                Some(
                    KnotCredentials::new()
                        .host("localhost")
                        .port(2222)
                        .username("root")
                        .auth_password(Password::new("Auth for localhost").prompt()?)
                        .limit(100),
                ),
            )
            .await?;
            {
                let pool = remote.resources.ssh.clone().unwrap();
                upload_and_prepare_server(pool, &fs::read("./target/release/knot")?).await?;
            }
            knots.add_remote(remote, Behavior::default());

            let start_time = Instant::now();
            let source_fut = knots.source.set_folder(main_config.clone());
            let remotes_fut = KnotManager::update_remotes(&mut knots.remotes, main_config);
            tokio::try_join!(source_fut, remotes_fut)?;
            println!("Update took: {:0.2?}", start_time.elapsed());

            for remote in &knots.remotes {
                let source = &knots.source;
                source.sync(remote).await?;
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
    };
    Ok(())
}
