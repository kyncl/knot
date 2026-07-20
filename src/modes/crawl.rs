use std::{path::PathBuf, sync::Arc};

use crate::{
    cli::StructFormat::{self, Binary, Json},
    configuration::MainConfig,
    knot::{Knot, KnotType},
    utils::compression::compress_data,
};
use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD};

/// This will start when
/// $ `knot crawl`
pub async fn crawl(
    format: StructFormat,
    compress: bool,
    crawl_path: PathBuf,
    main_config: MainConfig,
) -> Result<()> {
    let main_config = Arc::new(main_config);
    let knot = Knot::new(&KnotType::Local, crawl_path, None).await?;
    let folder = knot.crawl_dir(main_config).await?;
    match format {
        Json => {
            if compress {
                let data = serde_json::to_string(&folder)?;
                let compressed_data = compress_data(data.as_bytes(), 5)?;
                let encoded = STANDARD.encode(compressed_data);
                println!("{encoded}");
            } else {
                let data = serde_json::to_string_pretty(&folder)?;
                print!("{data}")
            }
        }
        Binary => {
            let data = rkyv::to_bytes::<rkyv::rancor::Error>(&folder)
                .map_err(|e| anyhow!("Failed to serialize payload with rkyv: {e}"))?;
            if compress {
                let compressed_data = compress_data(&data, 5)?;
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
