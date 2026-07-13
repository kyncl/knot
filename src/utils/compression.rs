use anyhow::Result;

pub fn compress_data(raw_data: &[u8], compression_level: i32) -> Result<Vec<u8>> {
    let compressed_data: Vec<u8> = zstd::encode_all(&raw_data[..], compression_level)?;
    Ok(compressed_data)
}

pub fn decompress_data(compressed_data: &[u8]) -> Result<Vec<u8>> {
    let original_data: Vec<u8> = zstd::decode_all(&compressed_data[..])?;
    Ok(original_data)
}
