use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::SystemTime,
};

use anyhow::Result;
use ignore::{WalkBuilder, WalkState};
use xxhash_rust::xxh3::Xxh3;

use crate::{BUFFER_SIZE, configuration::MainConfig, ignorer::should_ignore, knot::file::KnotFile};

pub fn local_file_crawler(folder: &PathBuf, config: Arc<MainConfig>) -> Result<Vec<KnotFile>> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let size_limit = config.performance.size_limit;

    let count = thread::available_parallelism()?.get();
    let walker = WalkBuilder::new(folder).threads(count).build_parallel();

    walker.run(|| {
        let results = Arc::clone(&results);
        let config = config.clone();
        let ignorer = config.global.ignorer.clone();

        Box::new(move |result| {
            let entry = match result {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };

            let path = entry.path();
            if should_ignore(path, folder, &ignorer) {
                return WalkState::Continue;
            }

            if let Ok(metadata) = entry.metadata() {
                let is_dir = metadata.is_dir();
                let mtime = metadata
                    .modified()
                    .unwrap_or(SystemTime::UNIX_EPOCH)
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let mut knot_file = KnotFile {
                    path: path.to_path_buf(),
                    mtime,
                    is_dir,
                    content_hash: None,
                };

                if is_dir {
                    results.lock().unwrap().push(knot_file);
                    return WalkState::Continue;
                }

                if metadata.is_file()
                    && (config.performance.allow_size_limit && metadata.len() <= size_limit
                        || !config.performance.allow_size_limit)
                    && let Ok(hash) = process_file(path, Some(metadata.len() as usize))
                {
                    knot_file.content_hash = Some(hash);
                    results.lock().unwrap().push(knot_file);
                }
            }

            WalkState::Continue
        })
    });
    let final_results = Arc::into_inner(results).unwrap().into_inner()?;
    Ok(final_results)
}

fn process_file<T>(path: T, file_size: Option<usize>) -> Result<u64>
where
    T: AsRef<Path>,
{
    let mut file = File::open(path)?;
    let mut hasher = Xxh3::new();

    let buffer_size = {
        if let Some(file_size) = file_size {
            if file_size < 16 * 1024 {
                file_size
            } else if file_size < 64 * 1024 {
                32 * 1024
            } else {
                64 * 1024
            }
        } else {
            BUFFER_SIZE as usize
        }
    };
    let mut buffer = vec![0; buffer_size];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hasher.digest())
}
