use crate::{
    BUFFER_SIZE_TRANSFER, TEMPORAL_SUFFIX,
    cli::subcommands::file_system::FileSubcommand,
    knot::{Knot, KnotType},
    utils::compression::Compressions,
};
use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD};
use parse_size::parse_size;
use std::{
    fs::{self, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write, stdin, stdout},
    path::{Path, PathBuf},
};
use tar::{Archive, Builder};
use zstd::{Decoder, Encoder};

pub async fn handle_files(cmd: FileSubcommand) -> Result<()> {
    let knot = Knot::new(KnotType::Local, PathBuf::new(), None).await?;
    match cmd {
        FileSubcommand::Write { path, data, offset } => {
            let decoded_bytes = STANDARD
                .decode(data.trim())
                .map_err(|e| anyhow!("Failed to decode Base64 data: {}", e))?;
            knot.write_at(&path, &decoded_bytes, offset).await?;
            println!("Successfully wrote data.");
        }
        FileSubcommand::WriteStream {
            path,
            temporal_path,
            expected_size,
        } => {
            let target_path = temporal_path.as_deref().unwrap_or(&path);
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(target_path)
                .map_err(|e| anyhow!("Failed to open destination file: {}", e))?;
            let mut reader = BufReader::with_capacity(BUFFER_SIZE_TRANSFER, io::stdin().lock());
            let mut writer = BufWriter::with_capacity(BUFFER_SIZE_TRANSFER, file);

            let bytes_copied = io::copy(&mut reader, &mut writer)
                .map_err(|e| anyhow!("Failed during stream write: {}", e))?;
            writer.flush()?;
            let file = writer.into_inner()?;
            file.sync_all()
                .map_err(|e| anyhow!("Failed to sync file to disk: {}", e))?;
            drop(file);
            if let Some(expected) = expected_size
                && bytes_copied != parse_size(&expected).unwrap_or(bytes_copied)
            {
                if let Some(temp_path) = &temporal_path {
                    let _ = fs::remove_file(temp_path);
                }
                return Err(anyhow!(
                    "Stream interrupted! Received {} bytes, expected {} bytes.",
                    bytes_copied,
                    expected
                ));
            }
            if let Some(temp_path) = temporal_path {
                fs::rename(temp_path, path)?;
            }
        }
        FileSubcommand::ReadStream { path } => {
            let file = std::fs::File::open(path)
                .map_err(|e| anyhow!("Failed to open source file: {}", e))?;
            let mut reader = BufReader::with_capacity(BUFFER_SIZE_TRANSFER, file);
            let mut writer = BufWriter::with_capacity(BUFFER_SIZE_TRANSFER, io::stdout().lock());
            io::copy(&mut reader, &mut writer)
                .map_err(|e| anyhow!("Failed during stream read: {}", e))?;
            writer.flush()?;
        }
        FileSubcommand::EmptyWrite { path, data } => {
            let decoded_bytes = STANDARD
                .decode(data.trim())
                .map_err(|e| anyhow!("Failed to decode Base64 data: {}", e))?;
            knot.overwrite(&path, &decoded_bytes).await?;
            println!("Overwrote file successfully.");
        }
        FileSubcommand::ReadInterval { path, start, end } => {
            let bytes = knot.read_range(&path, start..end).await?;
            let encoded = STANDARD.encode(&bytes);
            println!("{}", encoded);
        }
        FileSubcommand::ReadFull { path } => {
            let bytes = knot.read_all(&path).await?;
            let encoded = STANDARD.encode(&bytes);
            println!("{}", encoded);
        }
        FileSubcommand::Empty { path } => {
            knot.truncate(&path).await?;
            println!("File emptied successfully.");
        }
        FileSubcommand::Rename { old_path, new_path } => {
            knot.rename(&old_path, &new_path).await?;
            println!("File renamed successfully.");
        }
        FileSubcommand::Delete { path } => {
            knot.delete(path).await?;
            println!("File deleted.");
        }
        FileSubcommand::Create { path } => {
            knot.create(&path).await?;
            println!("Empty file created.");
        }
        FileSubcommand::CreateDir { path } => {
            knot.mkdir(&path).await?;
            println!("Directory created.");
        }
        FileSubcommand::CreateDirs { path } => {
            knot.mkdir_batch(path).await?;
        }
        FileSubcommand::WriteBatchStream {
            root_path,
            compression,
        } => {
            fs::create_dir_all(&root_path)?;
            let random_suffix = format!("{:016x}", rand::random::<u64>());
            let staging_dir = root_path.join(format!(".staging_{random_suffix}{TEMPORAL_SUFFIX}"));
            fs::create_dir_all(&staging_dir)?;

            let unpack_result = (|| -> anyhow::Result<()> {
                let stdin = stdin().lock();
                if compression == Compressions::Zstd {
                    let zstd_decoder = Decoder::new(stdin)?;
                    let mut archive = Archive::new(zstd_decoder);
                    archive.unpack(&staging_dir)?;
                } else {
                    let mut archive = Archive::new(stdin);
                    archive.unpack(&staging_dir)?;
                }
                Ok(())
            })();

            if let Err(e) = unpack_result {
                let _ = fs::remove_dir_all(&staging_dir);
                return Err(e);
            }
            atomic_commit(&staging_dir, &root_path)?;
        }
        FileSubcommand::ReadBatchStream {
            root_path,
            compression,
        } => {
            let stdin = stdin().lock();
            let stdout = stdout().lock();
            let lines = BufReader::new(stdin).lines();

            let pack_files = |mut archive: Builder<&mut dyn Write>| -> Result<()> {
                for line in lines {
                    let rel_path_str = line?;
                    if rel_path_str.trim().is_empty() {
                        continue;
                    }
                    let full_path = root_path.join(&rel_path_str);
                    if full_path.is_dir() {
                        archive.append_dir_all(&rel_path_str, &full_path)?;
                    } else {
                        let mut file = std::fs::File::open(&full_path)?;
                        archive.append_file(&rel_path_str, &mut file)?;
                    }
                }
                archive.finish()?;
                Ok(())
            };

            if compression == Compressions::Zstd {
                let mut zstd_encoder = Encoder::new(stdout, 3)?;
                let archive = Builder::new(&mut zstd_encoder as &mut dyn Write);
                pack_files(archive)?;
                let _ = zstd_encoder.finish()?;
            } else {
                let mut out = stdout;
                let archive = Builder::new(&mut out as &mut dyn Write);
                pack_files(archive)?;
                out.flush()?;
            }
        }
    }
    Ok(())
}

/// Will try safely commit all files from staging into root_path
pub fn atomic_commit(staging_dir: &Path, root_path: &Path) -> Result<()> {
    fs::create_dir_all(root_path)?;

    if fs::rename(staging_dir, root_path).is_ok() {
        return Ok(());
    }

    commit_directory_contents(staging_dir, root_path)?;
    let _ = fs::remove_dir_all(staging_dir);
    Ok(())
}

fn commit_directory_contents(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)?;
            commit_directory_contents(&src_path, &dst_path)?;
        } else {
            if dst_path.exists() {
                let _ = fs::remove_file(&dst_path);
            }
            fs::rename(&src_path, &dst_path)?;
        }
    }
    let _ = fs::remove_dir(src);
    Ok(())
}
