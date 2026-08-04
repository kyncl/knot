use anyhow::{Result, anyhow};
use std::{path::PathBuf, sync::Arc};

use crate::{
    ARCHIVE_PREFIX,
    cli::{
        resolvers::remote_indexing::resolve_remote_index,
        subcommands::archiving::ArchiveSubcommand,
        visualization::resolver::{ResolverFiles, resolve_files},
    },
    knot::{file::KnotFile, file_diffs::FileDiffs},
    modes::{archiving_local::list_archive, setup::setup, sync::add_unique_files},
    utils::{formatting::parse_human_time, paths::relative_path},
};

pub async fn handle_archiving(
    actions: Option<ArchiveSubcommand>,
    index: Option<usize>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let (main_config, knots) = setup(config_path).await?;
    let index = resolve_remote_index(index, &knots.remotes)?;
    let chosen = knots
        .remotes
        .get(index)
        .ok_or(anyhow!("Couldn't get the {index} remote knot"))?;
    let root_path = &chosen.knot.path;

    let source = knots.source.crawl_dir(Arc::clone(&main_config)).await?;
    let source_path = &knots.source.path;
    let remote_path = &chosen.knot.path;

    match actions {
        Some(ArchiveSubcommand::List {
            compress, format, ..
        }) => {
            let files = chosen.knot.crawl_dir(Arc::clone(&main_config)).await?;
            list_archive(files, format, compress)?;
            return Ok(());
        }
        // Root_path is saved in knot already
        Some(ArchiveSubcommand::Remove {
            target,
            force,
            older_than,
            ..
        }) => {
            if !target.is_empty() {
                chosen.knot.delete(target).await?;
            } else {
                let files = chosen.knot.crawl_dir(Arc::clone(&main_config)).await?;
                let archived: Vec<KnotFile> =
                    files
                        .into_iter()
                        .filter_map(|file| {
                            if file.path.file_name().is_some_and(|name| {
                                name.to_string_lossy().starts_with(ARCHIVE_PREFIX)
                            }) {
                                Some(file)
                            } else {
                                None
                            }
                        })
                        .collect();
                let time_limit = older_than
                    .as_ref()
                    .map(|t| parse_human_time(t))
                    .transpose()?;
                let to_delete: Vec<PathBuf> = archived
                    .into_iter()
                    .filter_map(|f| {
                        let should_delete = match time_limit {
                            Some(time) => f.mtime < time,
                            None => true,
                        };
                        if should_delete { Some(f.path) } else { None }
                    })
                    .collect();
                let should_do_it = if force {
                    true
                } else {
                    inquire::Confirm::new(&format!(
                        "Are you sure you want to delete {} files?\nThis will remove your archived files.",
                        to_delete.len()
                    ))
                    .with_default(false)
                    .prompt()?
                };
                if should_do_it {
                    chosen.knot.delete(to_delete).await?;
                    println!("Files were deleted successfully");
                }
            }
            return Ok(());
        }
        Some(ArchiveSubcommand::Recover { target, force, .. }) => {
            if !target.is_empty() {
                chosen.knot.recover_files(target, force).await?;
                let should_transfer = inquire::Confirm::new(
                    "Do you want to try transfer these files into your source?",
                )
                .with_default(false)
                .prompt()?;
                if should_transfer {
                    let remote = chosen.knot.crawl_dir(Arc::clone(&main_config)).await?;
                    let diffs = FileDiffs::new(&source, source_path, &remote, remote_path);
                    add_unique_files(
                        &diffs.remote_unique,
                        &chosen.knot.path,
                        &knots.source.path,
                        &chosen.knot,
                        &knots.source,
                        main_config.features.compress,
                    )
                    .await?;
                    println!("Transfer was successful");
                }
            } else {
                let files = chosen.knot.crawl_dir(Arc::clone(&main_config)).await?;
                let archived: Vec<PathBuf> =
                    files
                        .into_iter()
                        .filter_map(|file| {
                            if file.path.file_name().is_some_and(|name| {
                                name.to_string_lossy().starts_with(ARCHIVE_PREFIX)
                            }) {
                                Some(file.path)
                            } else {
                                None
                            }
                        })
                        .collect();
                chosen.knot.recover_files(archived, force).await?;
                let should_transfer = inquire::Confirm::new(
                    "Do you want to try transfer these files into your source?",
                )
                .with_default(false)
                .prompt()?;
                if should_transfer {
                    let remote = chosen.knot.crawl_dir(Arc::clone(&main_config)).await?;
                    let diffs = FileDiffs::new(&source, source_path, &remote, remote_path);
                    add_unique_files(
                        &diffs.remote_unique,
                        &chosen.knot.path,
                        &knots.source.path,
                        &chosen.knot,
                        &knots.source,
                        main_config.features.compress,
                    )
                    .await?;
                    println!("Transfer was successful");
                }
            }
            return Ok(());
        }
        Some(ArchiveSubcommand::Compress { dirs, files }) => {
            chosen.knot.archive_files(files, dirs).await?;
            println!("Archiving was successful");
            return Ok(());
        }
        None | Some(ArchiveSubcommand::Resolve { .. }) => {
            // Will go to resolve, which is under
        }
    }

    let mut did_chose = false;
    let force = true;
    let mut chosen_files: Vec<KnotFile> = vec![];
    let mut should_recrawl = true;
    loop {
        if should_recrawl {
            chosen_files = chosen.knot.crawl_dir(Arc::clone(&main_config)).await?;
            should_recrawl = false;
        }
        let archived: Vec<PathBuf> = chosen_files
            .iter()
            .filter_map(|file| {
                if file
                    .path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with(ARCHIVE_PREFIX))
                {
                    Some(PathBuf::from(relative_path(&file.path, root_path)))
                } else {
                    None
                }
            })
            .collect();

        if !chosen_files.is_empty() && archived.is_empty() {
            let diffs = FileDiffs::new(&source, source_path, &chosen_files, remote_path);
            if diffs.remote_unique.is_empty() {
                println!("No archived or remote unique files found");
                return Ok(());
            } else if !did_chose {
                if inquire::Confirm::new("Hmm, it seems there's no archived file but unique remote files. Do you want to recover them?")
                        .with_default(false)
                        .prompt()?
                {
                    add_unique_files(
                        &diffs.remote_unique,
                        &chosen.knot.path,
                        &knots.source.path,
                        &chosen.knot,
                        &knots.source,
                        main_config.features.compress,
                    )
                        .await?;
                    println!("If you have problems with file permissions with git, try `git checkout -- .`");
                }
                else {
                    println!("Nothing to resolve");
                }
                return Ok(());
            }
        }

        if archived.is_empty() {
            break;
        }

        let resolve = resolve_files(&archived, ResolverFiles::Archiving, Some(root_path))?;
        if resolve.first.is_empty() && resolve.second.is_empty() {
            break;
        }

        let recover_files: Vec<PathBuf> = resolve.first.iter().map(|p| root_path.join(p)).collect();
        let delete_files: Vec<PathBuf> = resolve.second.iter().map(|p| root_path.join(p)).collect();

        if !delete_files.is_empty() {
            did_chose = true;
            should_recrawl = true;
            chosen.knot.delete(delete_files).await?;
        }
        if !recover_files.is_empty() {
            did_chose = true;
            should_recrawl = true;
            chosen.knot.recover_files(recover_files, force).await?;
        }
    }

    if should_recrawl {
        chosen_files = chosen.knot.crawl_dir(Arc::clone(&main_config)).await?;
    }
    let diffs = FileDiffs::new(&source, source_path, &chosen_files, remote_path);
    add_unique_files(
        &diffs.remote_unique,
        &chosen.knot.path,
        &knots.source.path,
        &chosen.knot,
        &knots.source,
        main_config.features.compress,
    )
    .await?;

    Ok(())
}
