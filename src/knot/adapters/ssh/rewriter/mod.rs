use std::{path::Path, sync::Arc};

use anyhow::Result;

use crate::{
    BUFFER_SIZE_TRANSFER,
    knot::{
        Knot,
        adapters::{KnotAdapter, ssh::SSHAdapter},
        resources::KnotResourcers,
    },
};

pub mod stream_batch_local;
pub mod to_local;
impl SSHAdapter {
    pub async fn classic_rewrite(
        &self,
        foreign_knot: &Knot,
        temporal_file: &Path,
        foreign_path: &Path,
        path: &Path,
        resources: Arc<KnotResourcers>,
    ) -> Result<()> {
        let mut offset = 0u64;
        let chunk_size = BUFFER_SIZE_TRANSFER;

        loop {
            let chunk = self
                .read_range(
                    resources.clone(),
                    path,
                    offset..(offset + chunk_size as u64),
                )
                .await?;
            if chunk.is_empty() {
                break;
            }

            foreign_knot
                .write_at(&temporal_file, &chunk, offset)
                .await?;

            offset += chunk.len() as u64;
        }
        foreign_knot.rename(&temporal_file, foreign_path).await?;
        Ok(())
    }
}
