use std::path::Path;

use anyhow::Result;
use deadpool::managed::Object;
use std::io::Cursor;

use crate::{
    TEMPORAL_SUFFIX,
    connection::ssh::pool::SSHManager,
    knot::{Knot, file::KnotFile},
    modes::file::atomic_commit,
};

pub async fn stream_batch_local(
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
        "{bin} file read-batch-stream {compression_flag}--root-path '{}'",
        from_root.display()
    );

    let mut channel = session.session.channel_open_session().await?;
    channel.exec(true, cmd.as_bytes()).await?;
    let mut payload = String::new();
    for file in files {
        let rel_path = file.relative_path(from_root);
        payload.push_str(&rel_path);
        payload.push('\n');
    }
    channel.data(Cursor::new(payload.into_bytes())).await?;
    channel.eof().await?;

    tokio::fs::create_dir_all(&to_root).await?;
    let random_suffix = format!("{:016x}", rand::random::<u64>());
    let staging_dir = to_root.join(format!(".staging_{random_suffix}{TEMPORAL_SUFFIX}"));
    tokio::fs::create_dir_all(&staging_dir).await?;

    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(16);
    let staging_dir_clone = staging_dir.clone();

    let unpack_task = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        use tar::Archive;
        use zstd::stream::read::Decoder as ZstdDecoder;
        let sync_reader = ChannelReader::new(rx);

        let unpack = |reader: &mut dyn Read| -> anyhow::Result<()> {
            let mut archive = Archive::new(reader);
            archive.set_preserve_permissions(true);
            archive.set_overwrite(true);
            archive.unpack(&staging_dir_clone)?;
            Ok(())
        };

        if compress {
            let mut zstd_decoder = ZstdDecoder::new(sync_reader)?;
            unpack(&mut zstd_decoder)?;
        } else {
            let mut reader = sync_reader;
            unpack(&mut reader)?;
        }
        Ok(())
    });

    let tx_sender = Some(tx);
    let mut exit_code = None;
    let mut remote_stderr = String::new();

    while let Some(msg) = channel.wait().await {
        match msg {
            russh::ChannelMsg::Data { data } => {
                if let Some(sender) = tx_sender.as_ref() {
                    if sender.send(data.to_vec()).await.is_err() {
                        break;
                    }
                }
            }
            russh::ChannelMsg::ExtendedData { ext: 1, data } => {
                if let Ok(s) = std::str::from_utf8(&data) {
                    remote_stderr.push_str(s);
                }
            }
            russh::ChannelMsg::ExitStatus { exit_status } => {
                exit_code = Some(exit_status);
            }
            russh::ChannelMsg::ExitSignal {
                signal_name,
                core_dumped,
                error_message,
                ..
            } => {
                let _ = tokio::fs::remove_dir_all(&staging_dir).await;
                return Err(anyhow::anyhow!(
                    "Remote process was killed by signal: {:?} (core dumped: {}).\nMessage: {}\nRemote Stderr:\n{}",
                    signal_name,
                    core_dumped,
                    error_message,
                    remote_stderr
                ));
            }
            _ => {}
        }
    }
    drop(tx_sender);
    let unpack_result = unpack_task.await?;
    match exit_code {
        Some(0) => {
            unpack_result?;
            atomic_commit(&staging_dir, to_root)?;
            Ok(files.len())
        }
        Some(code) => {
            let _ = tokio::fs::remove_dir_all(&staging_dir).await;
            Err(anyhow::anyhow!(
                "Batch SSH download failed with exit code {code}.\nRemote Stderr:\n{remote_stderr}"
            ))
        }
        None => {
            let _ = tokio::fs::remove_dir_all(&staging_dir).await;
            Err(anyhow::anyhow!(
                "Channel closed before receiving exit status.\nRemote Stderr:\n{remote_stderr}"
            ))
        }
    }
}

use std::io::{self, Read};
use tokio::sync::mpsc::Receiver;

struct ChannelReader {
    rx: Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    offset: usize,
}

impl ChannelReader {
    fn new(rx: Receiver<Vec<u8>>) -> Self {
        Self {
            rx,
            buffer: Vec::new(),
            offset: 0,
        }
    }
}
impl Read for ChannelReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while self.offset >= self.buffer.len() {
            match self.rx.blocking_recv() {
                Some(new_buf) if !new_buf.is_empty() => {
                    self.buffer = new_buf;
                    self.offset = 0;
                }
                // Ignore empty buffers
                Some(_) => continue,
                // EOF
                None => {
                    return Ok(0);
                }
            }
        }

        let bytes_to_copy = std::cmp::min(buf.len(), self.buffer.len() - self.offset);
        buf[..bytes_to_copy]
            .copy_from_slice(&self.buffer[self.offset..self.offset + bytes_to_copy]);
        self.offset += bytes_to_copy;

        Ok(bytes_to_copy)
    }
}
