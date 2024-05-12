use crate::{ResourceIndex, ARK_FOLDER};
use anyhow::Result;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use log::info;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{fs, path::Path};

/// Watch the root path for changes and update the index
pub async fn watch_index<P: AsRef<Path>>(root_path: P) -> Result<()> {
    log::debug!(
        "Attempting to watch index at root path: {:?}",
        root_path.as_ref()
    );

    let root_path = fs::canonicalize(root_path.as_ref())?;
    let mut index = ResourceIndex::provide(&root_path)?;

    let (mut watcher, mut rx) = async_watcher()?;
    info!("Watching directory: {:?}", root_path);
    let config = Config::default();
    watcher.configure(config)?;
    watcher.watch(root_path.as_ref(), RecursiveMode::Recursive)?;
    info!("Started watcher with config: \n\t{:?}", config);

    let ark_folder = root_path.join(ARK_FOLDER);
    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                // If the event is a change in .ark folder, ignore it
                if event
                    .paths
                    .iter()
                    .any(|p| p.starts_with(&ark_folder))
                {
                    continue;
                }

                info!("Detected event: {:?}", event);
                index.update_all()?;
                index.store()?;
                info!("Index updated and stored");
            }
            Err(e) => log::error!("Error in watcher: {:?}", e),
        }
    }

    unreachable!("Watcher stream ended unexpectedly");
}

fn async_watcher(
) -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                if let Err(err) = tx.send(res).await {
                    log::error!("Error sending event: {:?}", err);
                }
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}
