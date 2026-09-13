use anyhow::Result;
use colored::Colorize;
use futures::{
    TryStreamExt,
    stream::{self, StreamExt},
};
use indicatif::HumanBytes;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
use tracing::debug;

use crate::{
    STABLE_CHANNELS_PER_SESSION,
    cli::{
        spinners::sync_load::{SyncLoading, clear_progress, use_terminal_spinner},
        visualization::resolver::{ResolverFiles, resolve_files},
    },
    configuration::MainConfig,
    knot::{Knot, file::KnotFile, file_diffs::FileDiffs, remote::RemoteKnot},
    utils::{
        behavior::{ConflictBehavior, UniqueBehavior},
        env::is_ci_environment,
    },
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

pub async fn sync(
    source: &Knot,
    remote: &RemoteKnot,
    config: Arc<MainConfig>,
    non_interactive: bool,
    sync_load: &SyncLoading,
) -> Result<()> {
    let remote_k = &remote.knot;
    let diff = source.difference(&remote.knot);
    if diff.source_unique.is_empty()
        && diff.remote_unique.is_empty()
        && diff.conflicts.is_empty()
        && diff.archived.is_empty()
    {
        sync_load.print(format!(
            " {}",
            "✔  Directory is synchronized with source".green()
        ))?;
        sync_load.cli_clear_and_hide()?;

        // Directories are synchronized
        return Ok(());
    }

    if !non_interactive {
        sync_load.cli_clear_and_hide()?;
        if diff.visualization() {
            sync_load.print_under(format!(" {}", "✖  Synchronization was canceled".red()))?;
            return Ok(());
        }
    }

    sync_load.cli_restore();
    let behavior = &remote.behavior;
    let compress = config.features.compress;
    let now = Instant::now();
    sync_load.ci_print("Syncing...");
    tokio::try_join!(
        handle_conflicts(source, remote_k, &diff.conflicts, &behavior.conflicts),
        handle_uniques(
            (source, remote_k),
            &diff,
            &behavior.uniques,
            compress,
            sync_load
        )
    )?;
    debug!("Synchronization took {:.2?}", now.elapsed());
    Ok(())
}

async fn handle_conflicts(
    source: &Knot,
    remote: &Knot,
    files: &[(KnotFile, KnotFile)],
    conflicts: &ConflictBehavior,
) -> Result<()> {
    let file_conflicts: Vec<&(KnotFile, KnotFile)> = files
        .iter()
        .filter(|(s, r)| !s.is_dir && !r.is_dir)
        .collect();
    if file_conflicts.is_empty() {
        return Ok(());
    }
    let limit = get_dynamic_io_limit(source, remote);
    match conflicts {
        ConflictBehavior::Ask => {
            let relative_files: Vec<PathBuf> = file_conflicts
                .iter()
                .filter_map(|(s, r)| {
                    if !s.is_dir && !r.is_dir {
                        Some(PathBuf::from(s.relative_path(&source.path)))
                    } else {
                        None
                    }
                })
                .collect();

            let resolved_files =
                resolve_files::<PathBuf>(&relative_files, ResolverFiles::SourceRemote, None)?;
            let source_iter = resolved_files.first.into_iter().map(|f| (f, true));
            let remote_iter = resolved_files.second.into_iter().map(|f| (f, false));

            stream::iter(source_iter.chain(remote_iter))
                .map(|(f, is_source_to_remote)| async move {
                    let s = source.path.join(&f);
                    let r = remote.path.join(&f);

                    if is_source_to_remote {
                        source.transfer_to(remote, &s, &r).await
                    } else {
                        remote.transfer_to(source, &r, &s).await
                    }
                })
                .buffer_unordered(limit)
                .try_collect::<Vec<()>>()
                .await?;
        }
        ConflictBehavior::Skip => {
            debug!("Skipping all conflicts...");
        }
        ConflictBehavior::Newer => {
            stream::iter(file_conflicts)
                .filter_map(|(s, r)| async move {
                    if s.mtime > r.mtime {
                        Some((source, remote, &s.path, &r.path))
                    } else if r.mtime > s.mtime {
                        Some((remote, source, &r.path, &s.path))
                    } else {
                        None
                    }
                })
                .map(|(from, to, src_path, dst_path)| async move {
                    from.transfer_to(to, src_path, dst_path).await
                })
                .buffer_unordered(limit)
                .try_collect::<Vec<()>>()
                .await?;
        }
        ConflictBehavior::Older => {
            stream::iter(file_conflicts)
                .filter_map(|(s, r)| async move {
                    if s.mtime < r.mtime {
                        Some((source, remote, &s.path, &r.path))
                    } else if r.mtime < s.mtime {
                        Some((remote, source, &r.path, &s.path))
                    } else {
                        None
                    }
                })
                .map(|(from, to, src_path, dst_path)| async move {
                    from.transfer_to(to, src_path, dst_path).await
                })
                .buffer_unordered(limit)
                .try_collect::<Vec<()>>()
                .await?;
        }
        ConflictBehavior::Source => {
            stream::iter(file_conflicts)
                .map(|(s, r)| async move { source.transfer_to(remote, &s.path, &r.path).await })
                .buffer_unordered(limit)
                .try_collect::<Vec<()>>()
                .await?;
        }
        ConflictBehavior::Remote => {
            stream::iter(file_conflicts)
                .map(|(s, r)| async move { remote.transfer_to(source, &r.path, &s.path).await })
                .buffer_unordered(limit)
                .try_collect::<Vec<()>>()
                .await?;
        }
    }

    Ok(())
}

async fn handle_uniques(
    // Source and Remote (In this order)
    (source, remote): (&Knot, &Knot),
    diffs: &FileDiffs,
    uniques: &UniqueBehavior,
    compress: bool,
    sync_load: &SyncLoading,
) -> Result<()> {
    match uniques {
        UniqueBehavior::Ask => {
            let source_paths: Vec<PathBuf> = diffs
                .source_unique
                .iter()
                .map(|file| file.path.clone())
                .collect();

            if !source_paths.is_empty() {
                let source_handler = resolve_files(
                    &source_paths,
                    ResolverFiles::UniqueHandle,
                    Some(&source.path),
                )?;

                let mut source_to_add = Vec::new();
                let mut source_to_delete = Vec::new();

                for file in &diffs.source_unique {
                    if source_handler.first.contains(&file.path) {
                        source_to_add.push(file);
                    } else if source_handler.second.contains(&file.path) {
                        source_to_delete.push(file);
                    }
                }

                if !source_to_add.is_empty() {
                    add_unique_files(
                        &source_to_add,
                        &diffs.source_root_path,
                        &diffs.remote_root_path,
                        source,
                        remote,
                        compress,
                        sync_load,
                    )
                    .await?;
                }

                if !source_to_delete.is_empty() {
                    execute_optimized_deletes(&mut source_to_delete, source).await?;
                }
            }

            let remote_paths: Vec<PathBuf> = diffs
                .remote_unique
                .iter()
                .map(|file| file.path.clone())
                .collect();

            if !remote_paths.is_empty() {
                let remote_handler = resolve_files(
                    &remote_paths,
                    ResolverFiles::UniqueHandle,
                    Some(&remote.path),
                )?;

                let mut remote_to_add = Vec::new();
                let mut remote_to_delete = Vec::new();

                for file in &diffs.remote_unique {
                    if remote_handler.first.contains(&file.path) {
                        remote_to_add.push(file);
                    } else if remote_handler.second.contains(&file.path) {
                        remote_to_delete.push(file);
                    }
                }

                if !remote_to_add.is_empty() {
                    add_unique_files(
                        &remote_to_add,
                        &diffs.remote_root_path,
                        &diffs.source_root_path,
                        remote,
                        source,
                        compress,
                        sync_load,
                    )
                    .await?;
                }

                if !remote_to_delete.is_empty() {
                    execute_optimized_deletes(&mut remote_to_delete, remote).await?;
                }
            }
        }
        UniqueBehavior::Skip => {
            debug!("Skipping all unique files...");
        }
        UniqueBehavior::Archive => {
            let now = Instant::now();
            add_unique_files(
                &diffs.source_unique,
                &diffs.source_root_path,
                &diffs.remote_root_path,
                source,
                remote,
                compress,
                sync_load,
            )
            .await?;
            debug!(
                "Adding {} unique files to remote {:?} took {:.2?}",
                diffs.source_unique.len(),
                remote.knot_type(),
                now.elapsed()
            );

            let mut to_archive: Vec<&KnotFile> = diffs.remote_unique.iter().collect();
            to_archive.sort_by_key(|f| std::cmp::Reverse(f.path.components().count()));

            let (archive_files, archive_dirs): (Vec<_>, Vec<_>) =
                to_archive.into_iter().partition(|file| !file.is_dir);

            let archive_files: Vec<PathBuf> =
                archive_files.into_iter().map(|f| f.path.clone()).collect();
            let archive_dirs: Vec<PathBuf> =
                archive_dirs.into_iter().map(|f| f.path.clone()).collect();
            let af_len = archive_files.len();
            let ad_len = archive_dirs.len();
            let now = Instant::now();
            remote.archive_files(archive_files, archive_dirs).await?;
            debug!(
                "Archiving {af_len} files and {ad_len} dirs took {:.2?}",
                now.elapsed()
            )
        }
        UniqueBehavior::OnlyAdd => {
            tokio::try_join!(
                add_unique_files(
                    &diffs.source_unique,
                    &diffs.source_root_path,
                    &diffs.remote_root_path,
                    source,
                    remote,
                    compress,
                    sync_load
                ),
                add_unique_files(
                    &diffs.remote_unique,
                    &diffs.remote_root_path,
                    &diffs.source_root_path,
                    remote,
                    source,
                    compress,
                    sync_load
                )
            )?;
        }
        UniqueBehavior::MirrorSource => {
            let mut remote_refs: Vec<&KnotFile> = diffs.remote_unique.iter().collect();
            execute_optimized_deletes(&mut remote_refs, remote).await?;
            add_unique_files(
                &diffs.source_unique,
                &diffs.source_root_path,
                &diffs.remote_root_path,
                source,
                remote,
                compress,
                sync_load,
            )
            .await?;
        }
        UniqueBehavior::MirrorRemote => {
            let mut source_refs: Vec<&KnotFile> = diffs.source_unique.iter().collect();
            execute_optimized_deletes(&mut source_refs, source).await?;
            add_unique_files(
                &diffs.remote_unique,
                &diffs.remote_root_path,
                &diffs.source_root_path,
                remote,
                source,
                compress,
                sync_load,
            )
            .await?;
        }
    }
    Ok(())
}

/// Batches and processes deletions cleanly.
/// If a parent directory is marked for deletion, it drops all internal files
/// from the pipeline since wiping the directory kills them all at once
async fn execute_optimized_deletes<F>(unique_files: &mut [F], target_knot: &Knot) -> Result<()>
where
    F: std::borrow::Borrow<KnotFile>,
{
    // Lexicographical sort guarantees parents come before children
    unique_files.sort_by(|a, b| {
        let a = a.borrow();
        let b = b.borrow();
        a.path.cmp(&b.path)
    });
    let mut optimized_deletes: Vec<PathBuf> = Vec::with_capacity(unique_files.len());
    let mut last_dir_path: Option<&Path> = None;

    for file in unique_files {
        let file = file.borrow();
        if let Some(parent_path) = last_dir_path
            && file.path.starts_with(parent_path)
        {
            continue;
        }
        if file.is_dir {
            last_dir_path = Some(&file.path);
        }
        optimized_deletes.push(file.path.clone());
    }

    target_knot.delete(optimized_deletes).await?;
    Ok(())
}

const SMALL_FILE_THRESHOLD: u64 = 512 * 1024; // 512 KB
const MAX_BATCH_BYTES: u64 = 16 * 1024 * 1024; // 16 MB per batch chunk
const MAX_BATCH_FILES: usize = 256; // Max files per open SSH channel

pub async fn add_unique_files<P, F>(
    unique_files: &[F],
    from_root_path: P,
    to_root_path: P,
    from_knot: &Knot,
    to_knot: &Knot,
    compress: bool,
    sync_load: &SyncLoading,
) -> Result<()>
where
    P: AsRef<Path>,
    F: std::borrow::Borrow<KnotFile>,
{
    let setup_time = Instant::now();
    let from_root = from_root_path.as_ref();
    let to_root = to_root_path.as_ref();

    let mut dirs_to_create: HashSet<PathBuf> = HashSet::with_capacity(unique_files.len());

    let mut large_files: Vec<&KnotFile> = Vec::new();
    let mut small_files: Vec<&KnotFile> = Vec::new();

    for file in unique_files {
        let file = file.borrow();
        let relative = file.relative_path(from_root);

        let clean_relative = relative.strip_prefix("/").unwrap_or(&relative);
        let foreign_path = to_root.join(clean_relative);

        if file.is_dir {
            if foreign_path != to_root && foreign_path.starts_with(to_root) {
                dirs_to_create.insert(foreign_path);
            }
        } else {
            if let Some(parent) = foreign_path.parent()
                && parent != to_root
                && parent.starts_with(to_root)
            {
                dirs_to_create.insert(parent.to_path_buf());
            }
            if file.size >= SMALL_FILE_THRESHOLD {
                large_files.push(file);
            } else {
                small_files.push(file);
            }
        }
    }

    let dirs_to_make = Instant::now();
    if !dirs_to_create.is_empty() {
        let spinner = sync_load.create_spinner("Creating directories...");
        let stop_term_spinner = use_terminal_spinner();

        let mut dirs: Vec<PathBuf> = dirs_to_create.into_iter().collect();
        dirs.sort_by_key(|d| d.components().count());
        to_knot.mkdir_batch(dirs).await?;

        if let Some(spinner) = spinner {
            spinner.finish_and_clear();
        }

        if let Some(term_spinner_stop) = stop_term_spinner {
            term_spinner_stop.store(true, Ordering::Relaxed);
        }
    }
    debug!("Dirs to make took: {:.2?}", dirs_to_make.elapsed());

    if small_files.is_empty() && large_files.is_empty() {
        sync_load.print_under("  No files to transfer")?;
        return Ok(());
    }

    let emoji = if is_ci_environment() {
        "📦"
    } else {
        &" 󰏗".yellow().to_string()
    };
    let threshold = HumanBytes(SMALL_FILE_THRESHOLD);
    let breakdown_msg = format!(
        "{emoji} Sync breakdown: {} small files (< {threshold}), {} large files (>= {threshold})",
        small_files.len(),
        large_files.len(),
    );
    sync_load.print_under(breakdown_msg)?;
    sync_load.ci_print("Working on file sending...");

    debug!("The set up to transfer took: {:.2?}", setup_time.elapsed());
    let total_tasks = large_files.len() + small_files.len();
    let pb = sync_load.create_sync_progress_bar(total_tasks);
    let done_tasks = Arc::new(AtomicUsize::new(0));

    let pb_large = pb.clone();
    let done_tasks_large = Arc::clone(&done_tasks);

    let large_transfer = stream::iter(large_files)
        .map(|file| {
            let pb_clone = pb_large.clone();
            let done_tasks_ref = Arc::clone(&done_tasks_large);
            async move {
                let path = file.path.clone();
                let relative = file.relative_path(from_root);
                let foreign_path = to_root.join(relative);
                from_knot.transfer_to(to_knot, &path, &foreign_path).await?;

                let current_done = done_tasks_ref.fetch_add(1, Ordering::SeqCst) + 1;
                if total_tasks > 0 {
                    let progress = (current_done as f64 / total_tasks as f64) * 100.0;
                    SyncLoading::inc(pb_clone.as_ref(), 1, progress as usize, total_tasks);
                }
                Ok::<(), anyhow::Error>(())
            }
        })
        .buffer_unordered(get_dynamic_io_limit(from_knot, to_knot))
        .try_collect::<Vec<()>>();

    let pb_small = pb.clone();
    let done_tasks_small = Arc::clone(&done_tasks);

    let small_transfer = async move {
        small_files.sort_unstable_by(|a, b| a.path.cmp(&b.path));
        let mut batches = Vec::new();
        let mut current_batch = Vec::with_capacity(MAX_BATCH_FILES);
        let mut current_bytes = 0;

        for file in small_files {
            current_bytes += file.size;
            current_batch.push(file);
            if current_bytes >= MAX_BATCH_BYTES || current_batch.len() >= MAX_BATCH_FILES {
                batches.push(std::mem::take(&mut current_batch));
                current_bytes = 0;
            }
        }
        if !current_batch.is_empty() {
            batches.push(current_batch);
        }
        let safe_ssh_batch_concurrency = get_dynamic_io_limit(from_knot, to_knot);

        stream::iter(batches)
            .map(|batch| {
                let pb_clone = pb_small.clone();
                let done_tasks_ref = Arc::clone(&done_tasks_small);
                async move {
                    let batch_size = from_knot
                        .transfer_batch(to_knot, &batch, from_root, to_root, compress)
                        .await?;

                    let current_done = done_tasks_ref
                        .fetch_add(batch_size as usize, Ordering::SeqCst)
                        + batch_size as usize;
                    if total_tasks > 0 {
                        let progress = (current_done as f64 / total_tasks as f64) * 100.0;
                        SyncLoading::inc(
                            pb_clone.as_ref(),
                            batch_size as u64,
                            progress as usize,
                            total_tasks,
                        );
                    }
                    Ok::<(), anyhow::Error>(())
                }
            })
            .buffer_unordered(safe_ssh_batch_concurrency)
            .try_collect::<Vec<()>>()
            .await?;
        Ok::<(), anyhow::Error>(())
    };

    tokio::try_join!(large_transfer, small_transfer)?;
    if let Some(pb) = pb {
        pb.finish_with_message("✔  Synchronization complete!".green().to_string());
    }
    clear_progress();

    sync_load.cli_clear()?;
    Ok(())
}
