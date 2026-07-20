use anyhow::{Result, anyhow};
use colored::Colorize;
use futures::{
    TryStreamExt,
    stream::{self, StreamExt},
};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};
use tracing::debug;

use crate::{
    ARCHIVE_PREFIX, STABLE_CHANNELS_PER_SESSION,
    knot::{Knot, file::KnotFile, file_diffs::FileDiffs, manager::RemoteKnot},
    utils::behavior::{ConflictBehavior, UniqueBehavior},
};

pub fn get_dynamic_io_limit(source: &Knot, remote: &Knot) -> usize {
    let source_size = get_dynamic_io_limit_single(source);
    let remote_size = get_dynamic_io_limit_single(remote);
    // Takes the smallest, because it's used in both situation and it's better that it will be
    // slower but more stable
    std::cmp::min(source_size, remote_size)
}

pub fn get_dynamic_io_limit_single(knot: &Knot) -> usize {
    if let Some(pool) = &knot.resources.ssh {
        let active_connections = pool.size;
        active_connections * STABLE_CHANNELS_PER_SESSION
    } else {
        32
    }
}

pub async fn sync(source: &Knot, remote: &RemoteKnot) -> Result<()> {
    let remote_k = &remote.knot;
    let diff = source.difference(&remote.knot);
    diff.visualization();
    let behavior = &remote.behavior;
    tokio::try_join!(
        handle_conflicts(source, remote_k, &diff.conflicts, &behavior.conflicts),
        handle_uniques(source, remote_k, &diff, &behavior.uniques)
    )?;
    Ok(())
}

async fn handle_conflicts(
    source: &Knot,
    remote: &Knot,
    files: &[(KnotFile, KnotFile)],
    conflicts: &ConflictBehavior,
) -> Result<()> {
    match conflicts {
        ConflictBehavior::Ask => {}
        ConflictBehavior::Skip => {
            debug!("Skipping all conflicts...");
        }
        ConflictBehavior::Newer
        | ConflictBehavior::Older
        | ConflictBehavior::Source
        | ConflictBehavior::Remote => {
            stream::iter(files)
                .filter(|(s, r)| futures::future::ready(!s.is_dir && !r.is_dir))
                .map(|(s_file, r_file)| async move {
                    let s_path = &s_file.path;
                    let r_path = &r_file.path;

                    match conflicts {
                        ConflictBehavior::Newer => {
                            if s_file.mtime > r_file.mtime {
                                source.transfer_to(remote, s_path, r_path).await
                            } else {
                                remote.transfer_to(source, r_path, s_path).await
                            }
                        }
                        ConflictBehavior::Older => {
                            if s_file.mtime < r_file.mtime {
                                source.transfer_to(remote, s_path, r_path).await
                            } else {
                                remote.transfer_to(source, r_path, s_path).await
                            }
                        }
                        ConflictBehavior::Source => {
                            source.transfer_to(remote, s_path, r_path).await
                        }
                        ConflictBehavior::Remote => {
                            remote.transfer_to(source, r_path, s_path).await
                        }
                        _ => Ok(()),
                    }
                })
                .buffer_unordered(get_dynamic_io_limit(source, remote))
                .try_collect::<Vec<()>>()
                .await?;
        }
    };

    Ok(())
}

async fn handle_uniques(
    source: &Knot,
    remote: &Knot,
    diffs: &FileDiffs,
    uniques: &UniqueBehavior,
) -> Result<()> {
    match uniques {
        UniqueBehavior::Ask => {}
        UniqueBehavior::Skip => {
            debug!("Skipping all unique files...");
        }
        UniqueBehavior::Archive => {
            add_unique_files(
                &diffs.source_unique,
                &diffs.source_root_path,
                &diffs.remote_root_path,
                source,
                remote,
            )
            .await?;

            let mut to_archive = diffs.remote_unique.clone();
            to_archive.sort_by_key(|f| std::cmp::Reverse(f.path.components().count()));

            stream::iter(to_archive)
                .map(|unique| async move {
                    let parent = unique
                        .path
                        .parent()
                        .ok_or_else(|| anyhow!("Couldn't get parent of {}", unique.path.display()))?
                        .to_path_buf();
                    let name = format!("{ARCHIVE_PREFIX}{}", unique.name()?);
                    remote.rename(&unique.path, &parent.join(name)).await
                })
                .buffer_unordered(get_dynamic_io_limit(source, remote))
                .collect::<Vec<Result<()>>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
        }
        UniqueBehavior::OnlyAdd => {
            tokio::try_join!(
                add_unique_files(
                    &diffs.source_unique,
                    &diffs.source_root_path,
                    &diffs.remote_root_path,
                    source,
                    remote
                ),
                add_unique_files(
                    &diffs.remote_unique,
                    &diffs.remote_root_path,
                    &diffs.source_root_path,
                    remote,
                    source
                )
            )?;
        }
        UniqueBehavior::MirrorSource => {
            execute_optimized_deletes(&diffs.remote_unique, remote).await?;
            add_unique_files(
                &diffs.source_unique,
                &diffs.source_root_path,
                &diffs.remote_root_path,
                source,
                remote,
            )
            .await?;
        }
        UniqueBehavior::MirrorRemote => {
            execute_optimized_deletes(&diffs.source_unique, source).await?;
            add_unique_files(
                &diffs.remote_unique,
                &diffs.remote_root_path,
                &diffs.source_root_path,
                remote,
                source,
            )
            .await?;
        }
    }
    Ok(())
}

/// Batches and processes deletions cleanly.
/// If a parent directory is marked for deletion, it drops all internal files
/// from the pipeline since wiping the directory kills them all at once
async fn execute_optimized_deletes(unique_files: &[KnotFile], target_knot: &Knot) -> Result<()> {
    let mut targets = unique_files.to_vec();
    // Lexicographical sort guarantees parents come before children
    targets.sort_by(|a, b| a.path.cmp(&b.path));
    let mut optimized_deletes: Vec<KnotFile> = Vec::with_capacity(targets.len());
    let mut last_dir_path: Option<PathBuf> = None;

    for file in targets {
        if let Some(ref parent_path) = last_dir_path {
            if file.path.starts_with(parent_path) {
                continue;
            }
        }
        if file.is_dir {
            last_dir_path = Some(file.path.clone());
        }
        optimized_deletes.push(file);
    }

    stream::iter(optimized_deletes)
        .map(|item| async move { target_knot.delete(&item.path).await })
        .buffer_unordered(get_dynamic_io_limit_single(target_knot))
        .collect::<Vec<Result<()>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

async fn add_unique_files<P>(
    unique_files: &[KnotFile],
    from_root_path: P,
    to_root_path: P,
    from_knot: &Knot,
    to_knot: &Knot,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let from_root_path = from_root_path.as_ref();
    let to_root_path = to_root_path.as_ref();
    let mut dirs_to_create: HashSet<PathBuf> = HashSet::with_capacity(unique_files.len());
    let mut files: Vec<KnotFile> = Vec::with_capacity(unique_files.len());

    for file in unique_files {
        let relative = file.relative_path(from_root_path);
        let foreign_path = to_root_path.join(relative);

        if file.is_dir {
            dirs_to_create.insert(foreign_path);
        } else {
            if let Some(parent) = foreign_path.parent() {
                dirs_to_create.insert(parent.to_path_buf());
            }
            files.push(file.clone());
        }
    }
    if !dirs_to_create.is_empty() {
        let dirs: Vec<PathBuf> = dirs_to_create.into_iter().collect();
        to_knot.mkdir_batch(dirs).await?;
    }

    let pb = create_sync_progress_bar(files.len());
    stream::iter(files)
        .map(|file| {
            let pb_clone = pb.clone();
            async move {
                let path = file.path.clone();
                let relative = file.relative_path(from_root_path);
                let foreign_path = to_root_path.join(relative);
                debug!("Rewriting file: {foreign_path:#?}");
                from_knot.transfer_to(to_knot, &path, &foreign_path).await?;
                pb_clone.inc(1);
                Ok::<(), anyhow::Error>(())
            }
        })
        .buffer_unordered(get_dynamic_io_limit(from_knot, to_knot))
        .try_collect::<Vec<()>>()
        .await?;
    pb.finish_with_message("✔  Sync complete!".green().to_string());
    Ok(())
}

pub fn create_sync_progress_bar(total_tasks: usize) -> ProgressBar {
    let pb = ProgressBar::new(total_tasks as u64);
    pb.set_style(
        ProgressStyle::with_template(
            " {spinner:.green} [{elapsed_precise}] [{bar:25.cyan/blue}] {pos}/{len} ({per_sec}, ETA {eta}) {wide_msg}",
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏ ")
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
