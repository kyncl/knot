use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Instant, SystemTime},
};

use crate::{
    ARCHIVE_PREFIX, BUFFER_SIZE, TEMPORAL_SUFFIX,
    configuration::MainConfig,
    ignorer::normalize_pattern,
    knot::file::{KnotFile, load_cache, save_cache},
};
use anyhow::Result;
use ignore::{WalkBuilder, WalkState, overrides::OverrideBuilder};
use tracing::{debug, warn};
use xxhash_rust::xxh3::Xxh3;

pub fn local_file_crawler(folder: &Path, config: Arc<MainConfig>) -> Result<Vec<KnotFile>> {
    let crawler_time = Instant::now();
    let (tx, rx) = mpsc::channel();
    let size_limit = config.performance.size_limit;
    let count = thread::available_parallelism()?.get();

    let cache = if config.features.caching {
        let now = Instant::now();
        let cache = load_cache(folder);
        debug!("Cache was loaded in {:.2?}", now.elapsed());
        cache
    } else {
        None
    }
    .map(Arc::new);
    let should_cache = Arc::new(AtomicBool::new(false));

    let mut builder = WalkBuilder::new(folder);
    builder
        .threads(count)
        .hidden(false)
        .ignore(false)
        .parents(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false);

    let mut ov_builder = OverrideBuilder::new(folder);
    let _ = ov_builder.add("!.git");
    let _ = ov_builder.add("!.git/**");

    if !config.global.ignore_patterns.is_empty() {
        let mut ov_builder = OverrideBuilder::new(folder);
        for pattern in &config.global.ignore_patterns {
            if let Some(normalized_pattern) = normalize_pattern(pattern)
                && let Err(e) = ov_builder.add(&normalized_pattern.override_fmt)
            {
                eprintln!("Warning: Invalid ignore pattern `{pattern}`: {e}");
            }
        }
        if let Ok(overrides) = ov_builder.build() {
            builder.overrides(overrides);
        }
    }

    let walker = builder.build_parallel();
    walker.run(|| {
        let tx = tx.clone();
        let config = config.clone();
        let cache = cache.clone();
        let should_cache = Arc::clone(&should_cache);

        Box::new(move |result| {
            let entry = match result {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };
            let absolute_path = entry.path().to_path_buf();
            let cache = cache.clone();

            if let Some(file_name) = absolute_path.file_name().and_then(|n| n.to_str())
                && file_name.ends_with(TEMPORAL_SUFFIX)
            {
                if let Ok(metadata) = std::fs::metadata(&absolute_path)
                    && !metadata.is_dir()
                    && let Err(err) = std::fs::remove_file(&absolute_path)
                {
                    debug!("Couldn't remove temporal file due to: {err}");
                } else if let Ok(metadata) = std::fs::metadata(&absolute_path)
                    && metadata.is_dir()
                    && let Err(err) = std::fs::remove_dir_all(&absolute_path)
                {
                    debug!("Couldn't remove temporal dir due to: {err}");
                } else {
                    debug!("Successfully removed {absolute_path:?}");
                }
                return WalkState::Continue;
            }

            if let Ok(metadata) = entry.metadata() {
                let is_dir = metadata.is_dir();
                let size = metadata.len();
                let mtime = metadata
                    .modified()
                    .unwrap_or(SystemTime::UNIX_EPOCH)
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;

                let mut knot_file = KnotFile {
                    path: absolute_path,
                    mtime,
                    size,
                    is_dir,
                    content_hash: None,
                };
                let is_archive = entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(ARCHIVE_PREFIX);

                let can_send = if metadata.is_dir() {
                    true
                } else if metadata.is_file()
                    && (!config.performance.allow_size_limit || knot_file.size <= size_limit)
                {
                    let path_cow = knot_file.path.to_string_lossy();
                    let path_slice: &str = path_cow.as_ref();

                    let cached_hash = if let Some(cache) = cache
                        && let Some(cached_file) = cache.get(path_slice)
                        && cached_file.mtime == knot_file.mtime
                    {
                        cached_file.content_hash
                    } else {
                        None
                    };

                    if let Some(hash) = cached_hash {
                        knot_file.content_hash = Some(hash);
                        true
                    } else if is_archive {
                        // Doesn't make sense to get hash of archived file
                        // It shouldn't be compared with other files
                        // Adding Some(0) so that the TUI can say "Hey, it's file"
                        knot_file.content_hash = Some(0);
                        true
                    } else if let Ok(hash) = process_file(&knot_file.path) {
                        // New hash, makes sense to store it in cache
                        should_cache.store(true, Ordering::Relaxed);
                        knot_file.content_hash = Some(hash);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if can_send && let Err(err) = tx.send(knot_file) {
                    eprintln!(
                        "Failed to send {:?} to mpsc channel due to {}",
                        err.0.path, err
                    );
                }
            }
            WalkState::Continue
        })
    });
    drop(tx);
    let final_results: Vec<KnotFile> = rx.into_iter().collect();
    let should_cache = should_cache.load(Ordering::Relaxed);
    let now = Instant::now();
    if config.features.caching
        && should_cache
        && let Err(err) = save_cache(folder, &final_results)
    {
        warn!("Couldn't save cache: {err}");
    }
    debug!("Cache was saved in {:.2?}", now.elapsed());
    debug!("Crawling took {:.2?}", crawler_time.elapsed());
    Ok(final_results)
}

fn process_file<T>(path: T) -> Result<u64>
where
    T: AsRef<Path>,
{
    let mut file = File::open(path)?;
    let mut hasher = Xxh3::new();
    let mut buffer = vec![0; BUFFER_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hasher.digest())
}
