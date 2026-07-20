use crate::{
    BUFFER_SIZE_TRANSFER,
    cli::FileSubcommand,
    knot::{Knot, KnotType},
};
use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{
    fs::{self, OpenOptions},
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
};

pub async fn handle_files(cmd: FileSubcommand) -> Result<()> {
    let knot = Knot::new(&KnotType::Local, PathBuf::new(), None).await?;
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
            io::copy(&mut reader, &mut writer)
                .map_err(|e| anyhow!("Failed during stream write: {}", e))?;
            writer.flush()?;
            let file = writer.into_inner()?;
            file.sync_all()
                .map_err(|e| anyhow!("Failed to sync file to disk: {}", e))?;
            drop(file);
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
            knot.delete(&path).await?;
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
    }
    Ok(())
}
