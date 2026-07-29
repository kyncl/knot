use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};
use tar::Builder;

use crate::{
    ARCHIVE_PREFIX, COMPRESSION_LEVEL,
    cli::{
        StructFormat::{self, Binary, Json},
        subcommands::archiving::ArchiveSubcommand,
        visualization::resolver::{ResolverFiles, resolve_files},
    },
    configuration::MainConfig,
    knot::{Knot, KnotType, file::KnotFile},
    utils::{compression::compress_data, formatting::parse_human_time, paths::relative_path},
};

/// Manipulates with your local archive
/// This is NOT making any communication with other knots unlike `handle_archiving`
pub async fn handle_local_archiving(actions: ArchiveSubcommand) -> Result<()> {
    let main_config = Arc::new(MainConfig::new());
    match actions {
        ArchiveSubcommand::Resolve { root_path } => {
            let knot = Knot::new(KnotType::Local, &root_path, None).await?;
            let files = knot.crawl_dir(main_config).await?;
            let archived: Vec<PathBuf> = files
                .into_iter()
                .filter_map(|file| {
                    if file.path.file_name().map_or(false, |name| {
                        name.to_string_lossy().starts_with(ARCHIVE_PREFIX)
                    }) {
                        Some(PathBuf::from(relative_path(file.path, &root_path)))
                    } else {
                        None
                    }
                })
                .collect();
            let resolve = resolve_files(&archived, ResolverFiles::Archiving, Some(&root_path))?;
            for r in resolve.first {
                let r = root_path.join(r);
                // Maybe adding option to do also it's children recursively
                // This may be useless, because this is primarily API for SSH connection
                handle_recover(&r, true).await?;
            }
            for r in resolve.second {
                let r = root_path.join(r);
                handle_remove(&r, None).await?;
            }
        }
        ArchiveSubcommand::List {
            root_path,
            format,
            compress,
        } => {
            let knot = Knot::new(KnotType::Local, root_path, None).await?;
            let files = knot.crawl_dir(main_config).await?;
            list_archive(files, format, compress)?;
        }
        ArchiveSubcommand::Recover {
            ref root_path,
            ref target,
            ..
        }
        | ArchiveSubcommand::Remove {
            ref root_path,
            ref target,
            ..
        } => {
            if let Some(root_path) = root_path {
                if !root_path.is_dir() {
                    return Err(anyhow!("Root path must be directory"));
                }

                let knot = Knot::new(KnotType::Local, root_path, None).await?;
                let files = knot.crawl_dir(Arc::clone(&main_config)).await?;
                let archived: Vec<PathBuf> = files
                    .into_iter()
                    .filter_map(|file| {
                        if file.path.file_name().map_or(false, |name| {
                            name.to_string_lossy().starts_with(ARCHIVE_PREFIX)
                        }) {
                            Some(file.path)
                        } else {
                            None
                        }
                    })
                    .collect();
                handle_recovery_remove(&archived, &actions).await?;
            } else if !target.is_empty() {
                let mut resolved_targets = Vec::new();
                for t in target {
                    if t.is_dir() {
                        let knot = Knot::new(KnotType::Local, t, None).await?;
                        let files = knot.crawl_dir(Arc::clone(&main_config)).await?;
                        for file in files {
                            if file.path.file_name().map_or(false, |name| {
                                name.to_string_lossy().starts_with(ARCHIVE_PREFIX)
                            }) {
                                resolved_targets.push(file.path);
                            }
                        }
                    } else {
                        resolved_targets.push(t.clone());
                    }
                }
                handle_recovery_remove(&resolved_targets, &actions).await?;
            } else {
                return Err(anyhow!("User didn't provide any root_path nor targets"));
            }
        }
        ArchiveSubcommand::Compress { dirs, files } => {
            archive_files(files, dirs).await?;
        }
    }
    Ok(())
}

pub fn list_archive(files: Vec<KnotFile>, format: StructFormat, compress: bool) -> Result<()> {
    let archived: Vec<KnotFile> = files
        .into_iter()
        .filter_map(|file| {
            if file.path.file_name().map_or(false, |name| {
                name.to_string_lossy().starts_with(ARCHIVE_PREFIX)
            }) {
                Some(file)
            } else {
                None
            }
        })
        .collect();
    match format {
        Json => {
            if compress {
                let data = serde_json::to_string(&archived)?;
                let compressed_data = compress_data(data.as_bytes(), COMPRESSION_LEVEL)?;
                let encoded = STANDARD.encode(compressed_data);
                println!("{encoded}");
            } else {
                let data = serde_json::to_string_pretty(&archived)?;
                println!("{data}")
            }
        }
        Binary => {
            let data = rkyv::to_bytes::<rkyv::rancor::Error>(&archived)
                .map_err(|e| anyhow!("Failed to serialize payload with rkyv: {e}"))?;
            if compress {
                let compressed_data = compress_data(&data, COMPRESSION_LEVEL)?;
                let encoded = STANDARD.encode(compressed_data);
                println!("{encoded}");
            } else {
                let encoded = STANDARD.encode(data);
                println!("{encoded}");
            }
        }
    };
    Ok(())
}

pub async fn archive_files(files: Vec<PathBuf>, dirs: Vec<PathBuf>) -> Result<()> {
    if !files.is_empty() {
        tokio::task::spawn_blocking(move || -> Result<()> {
            use std::fs::File;
            use tar::Header;
            for file in &files {
                let parent = file.parent().unwrap_or_else(|| Path::new(""));
                let name = file
                    .file_name()
                    .ok_or_else(|| anyhow!("Invalid file name: {:?}", file))?;

                let archive_name = format!("{ARCHIVE_PREFIX}{}.tar.zst", name.to_string_lossy());
                let archive_path = parent.join(archive_name);

                let tar_file = File::create(&archive_path)?;
                let mut mode_file = File::open(&file)?;
                let metadata = mode_file.metadata()?;

                let zstd_encoder = zstd::stream::Encoder::new(tar_file, 0)?;
                let mut builder = Builder::new(zstd_encoder);

                let mut header = Header::new_gnu();
                header.set_metadata(&metadata);

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    header.set_mode(mode);
                }

                builder.append_data(&mut header, name, &mut mode_file)?;

                let zstd_encoder = builder.into_inner()?;
                zstd_encoder.finish()?;

                std::fs::remove_file(&file)?;
            }
            Ok(())
        })
        .await??;
    }

    if !dirs.is_empty() {
        tokio::task::spawn_blocking(move || -> Result<()> {
            for dir in &dirs {
                let parent = dir.parent().unwrap_or_else(|| Path::new(""));
                let name = dir
                    .file_name()
                    .ok_or_else(|| anyhow!("Invalid directory name: {:?}", dir))?;

                let archive_name = format!("{ARCHIVE_PREFIX}{}.tar.zst", name.to_string_lossy());
                let archive_path = parent.join(archive_name);

                let tar_file = File::create(&archive_path)?;
                let zstd_encoder = zstd::stream::Encoder::new(tar_file, 0)?;
                let mut builder = Builder::new(zstd_encoder);
                builder.append_dir_all(name, &dir)?;
                builder.finish()?;
                let zstd_encoder = builder.into_inner()?;
                zstd_encoder.finish()?;
                std::fs::remove_dir_all(&dir)?;
            }
            Ok(())
        })
        .await??;
    }
    Ok(())
}

async fn handle_recovery_remove(paths: &[PathBuf], actions: &ArchiveSubcommand) -> Result<()> {
    for target in paths {
        let is_archive = target
            .file_name()
            .map(|name| name.to_string_lossy().starts_with(ARCHIVE_PREFIX))
            .unwrap_or(false);
        if !is_archive {
            eprintln!("{:#?} is not archived file; Skipping...", target);
            continue;
        }
        match actions {
            ArchiveSubcommand::Remove { older_than, .. } => {
                handle_remove(target, older_than.as_deref()).await?;
            }
            ArchiveSubcommand::Recover { force, .. } => {
                handle_recover(target, *force).await?;
            }
            _ => unreachable!(),
        };
    }
    Ok(())
}

async fn handle_remove(target: &Path, older_than: Option<&str>) -> Result<()> {
    let should_delete = if let Some(older_than) = older_than {
        let metadata = tokio::fs::metadata(target).await?;
        let file_time = metadata
            .created()
            .or_else(|_| metadata.modified())?
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;
        let time = parse_human_time(older_than)?;

        file_time < time
    } else {
        true
    };

    if should_delete {
        if target.is_dir() {
            tokio::fs::remove_dir_all(target).await?;
        } else {
            tokio::fs::remove_file(target).await?;
        }
    }
    Ok(())
}

pub async fn handle_recover(target: &Path, force: bool) -> Result<()> {
    let target = target.to_path_buf();
    let is_zstd = target.extension().map_or(false, |ext| ext == "zst");

    tokio::task::spawn_blocking(move || -> Result<()> {
        {
            let mut file = File::open(&target)?;
            let parent_dir = target.parent().unwrap_or_else(|| Path::new(""));
            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir)?;
            }

            let unpack_archive =
                |archive: &mut tar::Archive<&mut dyn std::io::Read>| -> Result<()> {
                    archive.set_overwrite(force);
                    archive.set_preserve_permissions(true);
                    archive.unpack(parent_dir)?;
                    Ok(())
                };

            if is_zstd {
                let mut zstd_decoder = zstd::stream::Decoder::new(file)?;
                let mut archive = tar::Archive::new(&mut zstd_decoder as &mut dyn std::io::Read);
                unpack_archive(&mut archive)?;
            } else {
                let mut archive = tar::Archive::new(&mut file as &mut dyn std::io::Read);
                unpack_archive(&mut archive)?;
            }
        }
        std::fs::remove_file(&target)?;
        Ok(())
    })
    .await??;
    Ok(())
}
