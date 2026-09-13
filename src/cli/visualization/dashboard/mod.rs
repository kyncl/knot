use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    KNOTS_CONFIGURATION,
    cli::{
        spinners::sync_load::SyncLoading,
        visualization::{
            dashboard::{
                drawing::DashDrawer, event_handler::DashEventHandler, file_node::UiFileNode,
            },
            rata_utils::{rata_clean, rata_init},
        },
    },
    configuration::{
        MainConfig,
        loader::{configuration::ConfigurationLoader, remote::RemoteKnotLoader},
    },
    knot::{file_diffs::FileDiffs, manager::KnotManager},
    modes::setup::{resolve_config_paths, setup},
};
use anyhow::{Result, anyhow};
use crossterm::event;
use ratatui::style::Color;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::sync::{RwLock, mpsc};
use tracing::debug;

pub mod drawing;
pub mod event_handler;
pub mod file_node;

pub async fn show_dashborad(config_path: Option<PathBuf>) -> Result<()> {
    let (config_path, config_dir) = resolve_config_paths(config_path.as_deref())?;
    let loaded_conf = ConfigurationLoader::load(&config_path)?;
    let loaded_knots = RemoteKnotLoader::load(config_dir.join(KNOTS_CONFIGURATION))?;

    let (main_config, knots) = setup(Some(&config_path))
        .await
        .map_err(|e| anyhow!("Setup failed: {e}"))?;

    let knots = Arc::new(RwLock::new(knots));
    let mut terminal = rata_init()?;
    let mut handler = DashEventHandler::default();

    let (tx, mut rx) = mpsc::channel(32);
    let mut diffs = Vec::with_capacity(loaded_knots.knots.len());
    let mut items_count =
        DashDrawer::redraw(&mut terminal, &handler, &loaded_conf, &loaded_knots, &diffs)?;

    folder_check(
        Arc::clone(&main_config),
        Arc::clone(&knots),
        tx.clone(),
        loaded_conf.source.path.clone(),
    );

    let mut last_tab = handler.selected_tab;
    let mut is_syncing = false;

    let mut saved_source_trees = Vec::with_capacity(loaded_knots.knots.len());
    let mut saved_remote_trees = Vec::with_capacity(loaded_knots.knots.len());
    loop {
        match rx.try_recv() {
            Ok(BackgroundMessage::DiffsLoaded(new_diffs, source_trees, remote_trees)) => {
                handler.init_tree(&source_trees, &remote_trees);
                saved_source_trees = source_trees;
                saved_remote_trees = remote_trees;
                diffs = new_diffs;
                handler.redraw = true;
            }
            Ok(BackgroundMessage::Error(err)) => {
                rata_clean()?;
                panic!("{err}");
            }
            _ => {}
        }

        if handler.redraw {
            if last_tab != handler.selected_tab {
                handler.init_tree(&saved_source_trees, &saved_remote_trees);
                last_tab = handler.selected_tab;
            }
            items_count =
                DashDrawer::redraw(&mut terminal, &handler, &loaded_conf, &loaded_knots, &diffs)?;
            handler.redraw = false;
        }

        // I'm sorry I tried everything and I mean
        // EVERYTHING. I changed to arcs, rwlocks even
        // cloning, but I just can't make it so that you can
        // stay in TUI and synchronize.
        // What's the problem? Well https://github.com/rust-lang/rust/issues/100013
        // Cargo says in future this will be fixed. Until then we must wait
        // The issue is from 2022.
        if handler.sync && !is_syncing {
            handler.sync = false;
            is_syncing = true;
            let mut did_sync = false;
            let knots_read = knots.read().await;
            if let Some(remote) = knots_read.remotes.get(handler.selected_tab) {
                rata_clean()?;
                let sync_load = SyncLoading::simple_cli();
                let source = &knots_read.source;
                source
                    .sync(remote, Arc::clone(&main_config), false, &sync_load)
                    .await?;
                did_sync = true;
            }
            drop(knots_read);
            if did_sync {
                terminal = rata_init()?;
                items_count = DashDrawer::redraw(
                    &mut terminal,
                    &handler,
                    &loaded_conf,
                    &loaded_knots,
                    &diffs,
                )?;
                folder_check(
                    Arc::clone(&main_config),
                    Arc::clone(&knots),
                    tx.clone(),
                    loaded_conf.source.path.clone(),
                );
                is_syncing = false;
            }
        }

        if event::poll(Duration::from_millis(10))? {
            let first_event = event::read()?;
            let mut events = vec![first_event];
            while event::poll(Duration::from_millis(0))? {
                events.push(event::read()?);
            }

            let k = knots.read().await;
            handler.handle(
                &events,
                &k.remotes,
                items_count,
                &saved_source_trees,
                &saved_remote_trees,
            );
            drop(k);

            if handler.end {
                break;
            }
        }
    }

    rata_clean()?;
    // So you don't have to wait a second
    // to end all operations like folder checking
    std::process::exit(0);
}

enum BackgroundMessage {
    DiffsLoaded(Vec<FileDiffs>, Vec<UiFileNode>, Vec<UiFileNode>),
    Error(String),
}

/// Folder check is done concurrently, meaning the rendering
/// shouldn't be frozen
fn folder_check(
    main_config: Arc<MainConfig>,
    knots: Arc<RwLock<KnotManager>>,
    tx: mpsc::Sender<BackgroundMessage>,
    source_path: PathBuf,
) {
    tokio::spawn(async move {
        let start_time = Instant::now();
        let mut k = knots.write().await;
        let source_fut = k.source.set_folder(Arc::clone(&main_config)).await;
        let remotes_fut =
            KnotManager::update_remotes(&mut k.remotes, Arc::clone(&main_config)).await;
        drop(k);

        if let (Ok(_), Ok(_)) = (&source_fut, &remotes_fut) {
            debug!("Background update took: {:0.2?}", start_time.elapsed());
            let k = knots.read().await;
            let mut diffs = Vec::with_capacity(k.remotes.len());
            for remote in k.remotes.iter() {
                let diff = k.source.difference(&remote.knot);
                diffs.push(diff);
            }
            drop(k);

            let (source_diff_nodes, remote_diff_nodes): (Vec<UiFileNode>, Vec<UiFileNode>) = diffs
                .par_iter()
                .map(|d| {
                    rayon::join(
                        || {
                            let unique_iter = d
                                .source_unique
                                .iter()
                                .map(|f| ("Unique", f, Color::LightGreen));
                            let conflict_iter = d
                                .conflicts
                                .iter()
                                .map(|(s, _)| ("Conflict", s, Color::LightRed));
                            let synced_iter =
                                d.synced.iter().map(|(s, _)| ("Synced", s, Color::DarkGray));
                            let combined_items =
                                unique_iter.chain(conflict_iter).chain(synced_iter);
                            UiFileNode::build_tree(combined_items, &source_path)
                        },
                        || {
                            let unique_iter = d
                                .remote_unique
                                .iter()
                                .map(|f| ("Unique", f, Color::LightGreen));
                            let conflict_iter = d
                                .conflicts
                                .iter()
                                .map(|(_, r)| ("Conflict", r, Color::LightRed));
                            let archive_iter = d
                                .archived
                                .iter()
                                .map(|r| ("Archived", r, Color::LightYellow));
                            let synced_iter =
                                d.synced.iter().map(|(_, r)| ("Synced", r, Color::DarkGray));

                            let combined_items = unique_iter
                                .chain(conflict_iter)
                                .chain(archive_iter)
                                .chain(synced_iter);
                            UiFileNode::build_tree(combined_items, &d.remote_root_path)
                        },
                    )
                })
                .unzip();

            let _ = tx
                .send(BackgroundMessage::DiffsLoaded(
                    diffs,
                    source_diff_nodes,
                    remote_diff_nodes,
                ))
                .await;
        } else {
            let source_msg = if let Err(err) = source_fut {
                format!("{err}")
            } else {
                "Unknown".to_string()
            };
            let remote_msg = if let Err(err) = remotes_fut {
                format!("{err}")
            } else {
                "Unknown".to_string()
            };
            let msg = format!(
                "Background update failed.\n\tSource cause: {source_msg}\n\tRemote cause: {remote_msg}"
            );
            let _ = tx.send(BackgroundMessage::Error(msg)).await;
        }
    });
}
