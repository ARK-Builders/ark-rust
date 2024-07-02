use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

use crate::index_registrar::provide_index;
use data_pdf::{render_preview_page, PDFQuality};

// This is where the `ResourceId` type is defined.
// Change this to use another type for the resource id if needed.
//
// We define it globally here so that it can be easily changed.
pub(crate) use dev_hash::Crc32 as ResourceId;

use fs_atomic_versions::app_id;
use fs_storage::ARK_FOLDER;

use anyhow::Result;

use chrono::{prelude::DateTime, Utc};

use clap::{CommandFactory, FromArgMatches};

use fs_extra::dir::{self, CopyOptions};

use home::home_dir;

use crate::{
    cli::Cli,
    commands::{
        file::File::{Append, Insert, Read},
        link::Link::{Create, Load},
        Commands::{Link, Storage, *},
    },
    models::{EntryOutput, Format, Sort},
};

use crate::error::AppError;

use util::{
    discover_roots, monitor_index, provide_root, read_storage_value,
    storages_exists, timestamp, translate_storage,
};

mod cli;
mod commands;
mod error;
mod index_registrar;
mod models;
mod util;

const ARK_CONFIG: &str = ".config/ark";
const ARK_BACKUPS_PATH: &str = ".ark-backups";
const ROOTS_CFG_FILENAME: &str = "roots";

struct StorageEntry {
    path: Option<PathBuf>,
    resource: Option<ResourceId>,
    content: Option<String>,
    tags: Option<Vec<String>>,
    scores: Option<u32>,
    datetime: Option<String>,
}

async fn run() -> Result<()> {
    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;
    match cli.command {
        Backup(backup) => backup.run()?,
        Collisions(collisions) => collisions.run()?,
        Monitor(monitor) => monitor.run()?,
        Render(render) => render.run()?,
        List(list) => list.run()?,
        Link { subcommand } => match subcommand {
            Create(create) => create.run().await?,
            Load(load) => load.run()?,
        },
        crate::commands::Commands::File { subcommand } => match subcommand {
            Append(append) => append.run()?,
            Insert(insert) => insert.run()?,
            Read(read) => read.run()?,
        },
        Storage { subcommand } => match subcommand {
            crate::commands::storage::Storage::List(list) => list.run()?,
        },
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().default_filter_or("info"),
    );

    let app_id_dir = home_dir().ok_or(AppError::HomeDirNotFound)?;
    let ark_dir = app_id_dir.join(".ark");
    if !ark_dir.exists() {
        std::fs::create_dir(&ark_dir)
            .map_err(|e| AppError::ArkDirectoryCreationError(e.to_string()))?;
    }

    println!("Loading app id at {}...", ark_dir.display());
    let _ = app_id::load(ark_dir)
        .map_err(|e| AppError::AppIdLoadError(e.to_string()))?;

    // Having a separate function for the main logic allows for easier
    // error handling and testing.
    if let Err(err) = run().await {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }

    Ok(())
}
