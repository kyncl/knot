use crate::{BUFFER_SIZE_TRANSFER, knot::Knot};
use anyhow::Result;
use std::path::Path;
use tokio::io::AsyncReadExt;

pub mod stream_batch_ssh;
pub mod to_ssh;

pub async fn classic_rewrite(
    foreign_knot: &Knot,
    local_path: &Path,
    temporal_file: &Path,
    foreign_path: &Path,
) -> Result<()> {
    let mut file = tokio::fs::File::open(local_path).await?;
    let mut buffer = vec![0; BUFFER_SIZE_TRANSFER];
    let mut current_offset = 0u64;
    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        foreign_knot
            .write_at(temporal_file, &buffer[..bytes_read], current_offset)
            .await?;
        current_offset += bytes_read as u64;
    }
    foreign_knot.rename(temporal_file, foreign_path).await
}
