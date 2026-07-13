use std::{path::PathBuf, sync::Arc};

use crate::{
    cli::StructFormat::{self, Json, Postcard},
    configuration::MainConfig,
    knot::{Knot, KnotType},
    utils::compression::compress_data,
};
use anyhow::Result;
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
    let knot = Knot::new(&KnotType::Local, crawl_path, None, main_config.clone()).await?;
    let folder = knot.get_folder(main_config).await?;
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
        Postcard => {
            let data = postcard::to_allocvec(&folder)?;
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
