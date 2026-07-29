use crate::{
    COMPRESSION_LEVEL,
    connection::ssh::pool::SSHManager,
    knot::{Knot, file::KnotFile},
};

use anyhow::Result;
use deadpool::managed::Object;
use std::io::{Cursor, Write};
use std::path::Path;
use tar::{EntryType, Header};
use tokio::io::AsyncReadExt;
use zstd::Encoder;

pub async fn stream_batch_ssh(
    files: &[KnotFile],
    from_root: &Path,
    to_root: &Path,
    foreign_knot: &Knot,
    foreign_session: Object<SSHManager>,
    compress: bool,
) -> Result<usize> {
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

    let compression_flag = if compress { "--compression zstd " } else { "" };
    let cmd = format!(
        "{bin} file write-batch-stream {compression_flag}--root-path '{}'",
        to_root.display()
    );

    let mut channel = session.session.channel_open_session().await?;
    channel.exec(true, cmd.as_bytes()).await?;

    const FLUSH_THRESHOLD: usize = 256 * 1024;
    let mut raw_tar_buf = Vec::with_capacity(FLUSH_THRESHOLD + 512 * 1024);
    let compressed_buf = Vec::with_capacity(FLUSH_THRESHOLD);
    let mut file_read_buf = Vec::new();

    let mut zstd_encoder = if compress {
        Some(Encoder::new(compressed_buf, COMPRESSION_LEVEL)?)
    } else {
        None
    };

    for file in files {
        let relative_path = file.relative_path(from_root);

        let mut local_file = tokio::fs::File::open(&file.path).await?;
        let metadata = local_file.metadata().await?;
        let file_mode = if cfg!(unix) {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode()
        } else {
            0o644
        };

        file_read_buf.clear();
        local_file.read_to_end(&mut file_read_buf).await?;

        let file_size = file_read_buf.len() as u64;
        let path_bytes = relative_path.as_bytes();

        // Handle Long Paths
        if path_bytes.len() > 100 {
            let mut long_header = Header::new_gnu();
            long_header.set_path("././@LongLink")?;
            long_header.set_size(path_bytes.len() as u64);
            long_header.set_mode(file_mode);
            long_header.set_entry_type(EntryType::GNULongName);
            long_header.set_cksum();
            raw_tar_buf.extend_from_slice(long_header.as_bytes());
            raw_tar_buf.extend_from_slice(path_bytes);
            let long_rem = (512 - (path_bytes.len() % 512)) % 512;
            if long_rem > 0 {
                raw_tar_buf.resize(raw_tar_buf.len() + long_rem, 0u8);
            }
        }

        // Handle Main Header
        let mut header = Header::new_gnu();
        if path_bytes.len() > 100 {
            header.set_path("longpath")?;
        } else {
            header.set_path(&relative_path)?;
        }
        header.set_size(file_size);
        header.set_mode(file_mode);
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                header.set_mtime(duration.as_secs());
            }
        }
        header.set_cksum();

        raw_tar_buf.extend_from_slice(header.as_bytes());
        raw_tar_buf.extend_from_slice(&file_read_buf);
        let remainder = (512 - (file_size % 512)) % 512;
        if remainder > 0 {
            raw_tar_buf.resize(raw_tar_buf.len() + remainder as usize, 0u8);
        }

        // Write data either through Zstd or directly to SSH channel
        if let Some(ref mut encoder) = zstd_encoder {
            encoder.write_all(&raw_tar_buf)?;
            raw_tar_buf.clear();

            if encoder.get_ref().len() >= FLUSH_THRESHOLD {
                encoder.flush()?;
                let out_bytes = encoder.get_mut();
                channel.data(Cursor::new(&out_bytes[..])).await?;
                out_bytes.clear();
            }
        } else if raw_tar_buf.len() >= FLUSH_THRESHOLD {
            channel.data(Cursor::new(&raw_tar_buf[..])).await?;
            raw_tar_buf.clear();
        }
    }

    // 1024B TAR EOF block
    raw_tar_buf.resize(raw_tar_buf.len() + 1024, 0u8);

    // Sending data through stream
    if let Some(mut encoder) = zstd_encoder {
        if !raw_tar_buf.is_empty() {
            encoder.write_all(&raw_tar_buf)?;
        }
        let final_buf = encoder.finish()?;
        if !final_buf.is_empty() {
            channel.data(Cursor::new(&final_buf)).await?;
        }
    } else if !raw_tar_buf.is_empty() {
        channel.data(Cursor::new(&raw_tar_buf[..])).await?;
    }

    channel.eof().await?;

    let mut exit_code = None;
    let mut remote_stderr = String::new();
    while let Some(msg) = channel.wait().await {
        match msg {
            russh::ChannelMsg::ExitStatus { exit_status } => {
                exit_code = Some(exit_status);
                break;
            }
            russh::ChannelMsg::Data { .. } => {}
            russh::ChannelMsg::ExtendedData { ext: 1, data } => {
                if let Ok(s) = std::str::from_utf8(&data) {
                    remote_stderr.push_str(s);
                }
            }
            _ => {}
        }
    }

    let _ = channel.close().await;

    match exit_code {
        Some(0) => Ok(files.len()),
        Some(code) => Err(anyhow::anyhow!(
            "Batch SSH upload failed with exit code {code}.\nRemote Stderr:\n{remote_stderr}"
        )),
        None => Err(anyhow::anyhow!(
            "Channel closed before receiving exit status.\nRemote Stderr:\n{remote_stderr}"
        )),
    }
}
