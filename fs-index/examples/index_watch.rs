use anyhow::Result;
use fs_index::watch::watch_index;
use std::{path::Path, thread};

/// Example demonstrating how to use fs_index to watch a directory for changes in a separate thread.
/// This automatically updates the index when changes are detected.
fn main() -> Result<()> {
    let root = Path::new("test-assets");

    let thread_handle = thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                if let Err(err) = watch_index(root).await {
                    eprintln!("Error in watching index: {:?}", err);
                }
            });
    });

    thread_handle
        .join()
        .expect("Failed to join thread");

    Ok(())
}
