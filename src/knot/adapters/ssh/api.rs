use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::{path::Path, sync::Arc};

use crate::{BUFFER_SIZE, connection::ssh::pool::SSHPool};

pub async fn test(pool: Arc<SSHPool>) -> Result<()> {
    let session = pool.try_get_session(3).await?;
    let msg = session.call("echo hello from SSH").await?;
    println!("{msg:?}");
    Ok(())
}

pub async fn remove<P>(pool: Arc<SSHPool>, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_string_lossy();
    let session = pool.try_get_session(3).await?;
    session.call(&format!("rm -rf {path}")).await?;
    Ok(())
}

pub async fn rename<P>(pool: Arc<SSHPool>, old_path: P, new_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let old = old_path.as_ref().to_string_lossy();
    let new = new_path.as_ref().to_string_lossy();
    let session = pool.try_get_session(3).await?;
    session.call(&format!("mv {old} {new}")).await?;
    Ok(())
}

pub async fn upload_and_prepare_server(
    pool: Arc<SSHPool>,
    local_binary_bytes: &[u8],
) -> Result<()> {
    let session = pool.try_get_session(3).await?;

    let remote_path = String::from("~/.local/bin/knot");
    println!("Creating remote directory...");
    let mut channel = session.session.channel_open_session().await?;
    channel.exec(true, format!("mkdir -p ~/.local/bin")).await?;
    while let Some(_) = channel.wait().await {}

    println!("Encoding binary to Base64...");
    let b64_encoded = STANDARD.encode(local_binary_bytes);

    println!("Uploading binary via Base64 stream...");
    let mut upload_channel = session.session.channel_open_session().await?;

    let cmd = format!("base64 -d > {}", remote_path);
    upload_channel.exec(true, cmd).await?;

    for chunk in b64_encoded.as_bytes().chunks(BUFFER_SIZE as usize) {
        upload_channel.data(chunk).await?;
    }
    upload_channel.eof().await?;
    while let Some(_) = upload_channel.wait().await {}
    println!("Upload complete.");
    println!("Setting executable permissions...");
    let mut chmod_channel = session.session.channel_open_session().await?;
    chmod_channel
        .exec(true, format!("chmod +x {}", remote_path))
        .await?;
    while let Some(_) = chmod_channel.wait().await {}
    println!("Remote server binary is ready to run!");
    Ok(())
}
