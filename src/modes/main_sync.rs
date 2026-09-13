use anyhow::{Result, anyhow};
use colored::*;
use futures::future;
use std::{sync::Arc, time::Instant};
use tracing::debug;

use crate::{
    cli::spinners::sync_load::{SyncLoading, SyncStatus},
    configuration::MainConfig,
    knot::{file::KnotFile, manager::KnotManager},
    utils::notifications::send_notification,
};

pub async fn main_sync(
    knots: &mut KnotManager,
    main_config: Arc<MainConfig>,
    source_files: Option<Vec<KnotFile>>,
    non_interactive: bool,
) -> Result<Vec<Result<bool>>> {
    let start_time = Instant::now();
    let source_fut = async {
        if let Some(files) = source_files {
            knots.source.files = files;
            Ok(())
        } else {
            knots
                .source
                .set_folder(Arc::clone(&main_config))
                .await
                .map_err(|e| anyhow!("Source setup failed: {e}"))
        }
    };
    let remotes_fut = async {
        KnotManager::update_remotes(&mut knots.remotes, Arc::clone(&main_config))
            .await
            .map_err(|e| anyhow!("Remote update failed: {e}"))
    };
    tokio::try_join!(source_fut, remotes_fut)?;
    debug!("Update took: {:0.2?}", start_time.elapsed());

    let statuses = if main_config.experimental.async_sync {
        let sync_load = &SyncLoading::basic();
        let sync_fut = knots.remotes.iter().enumerate().map(|(index, remote)| {
            let source = &knots.source;
            let config_clone = Arc::clone(&main_config);
            sync_load.ci_print("Doing remote Knot #{index} ...");
            async move {
                source
                    .sync(remote, config_clone, non_interactive, sync_load)
                    .await
                    .map_err(|e| anyhow!("Sync failed on remote #{index}: {e}"))
            }
        });
        future::join_all(sync_fut).await
    } else {
        let sync_load = SyncLoading::full_cli();
        let total_remotes = knots.remotes.len();
        let mut statuses = Vec::with_capacity(total_remotes);

        for (index, remote) in knots.remotes.iter().enumerate() {
            sync_load.update_node(index, total_remotes);
            let config_clone = Arc::clone(&main_config);
            sync_load.ci_print("Doing remote Knot #{index} ...");
            statuses.push(
                knots
                    .source
                    .sync(remote, config_clone, non_interactive, &sync_load)
                    .await
                    .map_err(|e| anyhow::anyhow!("Sync failed on remote #{index}: {e}")),
            );
        }

        let mut syncs = Vec::with_capacity(statuses.len());
        for status in &statuses {
            if let Ok(s) = status {
                if *s {
                    syncs.push(SyncStatus::Ok);
                } else {
                    syncs.push(SyncStatus::Canceled);
                }
            } else {
                syncs.push(SyncStatus::Err);
            }
        }
        sync_load.node_finish(total_remotes, &syncs);

        statuses
    };

    eprintln!(" {}", "<=> Synchronization process finished".blue());
    Ok(statuses)
}

pub fn handle_sync_notifications(statuses: &[Result<bool>]) {
    let status_len = statuses.len();
    let mut error_happened = 0;
    let mut error_msgs = String::new();

    for status in statuses {
        if let Err(err) = status {
            error_happened += 1;
            error_msgs.push_str(&format!("{err}\n"));
        }
    }

    if error_happened == 0 {
        send_notification(
            "Successful synchronization",
            "All knots were successfully synchronized",
        );
    } else if error_happened == status_len {
        send_notification("Synchronization fully failed", error_msgs);
    } else {
        send_notification("Partial failed synchronization", error_msgs);
    }
}
