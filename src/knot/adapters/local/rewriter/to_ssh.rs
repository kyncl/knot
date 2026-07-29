use std::path::Path;

use anyhow::{Result, anyhow};
use deadpool::managed::Object;
use tokio::io::AsyncReadExt;

use crate::{BUFFER_SIZE_TRANSFER, connection::ssh::pool::SSHManager, knot::Knot};

pub async fn stream_rewrite_ssh(
    path: &Path,
    foreign_knot: &Knot,
    foreign_session: Object<SSHManager>,
    temporal_file: &Path,
    foreign_path: &Path,
) -> Result<()> {
    let session = foreign_session;
    let bin = if let Some(bin) = foreign_knot.resources.ssh_executable.as_deref() {
        bin
    } else {
        if let Ok((code, _)) = session.call("knot -V").await
            && code == 0
        {
            "knot"
        } else {
            "./.local/bin/knot"
        }
    };
    let mut local_file = tokio::fs::File::open(path).await?;
    let file_size = local_file.metadata().await?.len();

    let safe_foreign = foreign_path.display().to_string().replace('\'', "'\\''");
    let safe_temporal = temporal_file.display().to_string().replace('\'', "'\\''");

    let cmd = format!(
        "{bin} file write-stream '{safe_foreign}' --temporal-path '{safe_temporal}' --expected-size {file_size}"
    );
    let mut channel = session.session.channel_open_session().await?;
    channel.exec(true, cmd.as_bytes()).await?;

    let mut buffer = vec![0; BUFFER_SIZE_TRANSFER];

    loop {
        let bytes_read = local_file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        channel.data(&buffer[..bytes_read]).await?;
    }
    channel.eof().await?;
    let mut exit_code = None;
    while let Some(msg) = channel.wait().await {
        if let russh::ChannelMsg::ExitStatus { exit_status } = msg {
            exit_code = Some(exit_status);
            break;
        }
    }

    match exit_code {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!("Streamlined upload failed with exit code {code}")),
        None => Err(anyhow!("Channel closed before receiving an exit status")),
    }
}
