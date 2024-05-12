use crate::AppError;
use anyhow::Result;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use data_resource::ResourceId;

use super::{
    entry::EntryOutput, format::Format, sort::Sort, storage::StorageType,
};

#[derive(Parser, Debug)]
#[clap(name = "ark-cli")]
#[clap(about = "Manage ARK tag storages and indexes", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Backup {
        #[clap(parse(from_os_str))]
        roots_cfg: Option<PathBuf>,
    },

    Collisions {
        #[clap(parse(from_os_str))]
        root_dir: Option<PathBuf>,
    },

    Monitor {
        #[clap(parse(from_os_str))]
        root_dir: Option<PathBuf>,
        interval: Option<u64>,
    },

    Render {
        #[clap(parse(from_os_str))]
        path: Option<PathBuf>,
        quality: Option<String>,
    },

    List {
        #[clap(parse(from_os_str))]
        root_dir: Option<PathBuf>,

        #[clap(
            long,
            short = 'i',
            long = "id",
            action,
            help = "Show entries' IDs"
        )]
        entry_id: bool,

        #[clap(
            long,
            short = 'p',
            long = "path",
            action,
            help = "Show entries' paths"
        )]
        entry_path: bool,

        #[clap(
            long,
            short = 'l',
            long = "link",
            action,
            help = "Show entries' links"
        )]
        entry_link: bool,

        #[clap(long, short, action)]
        modified: bool,

        #[clap(long, short, action)]
        tags: bool,

        #[clap(long, short, action)]
        scores: bool,

        #[clap(long, value_enum)]
        sort: Option<Sort>,

        #[clap(long)]
        filter: Option<String>,
    },

    #[clap(
        about = "Watch a directory for changes and update the index automatically"
    )]
    Watch {
        #[clap(
            help = "Path to the directory to watch for changes",
            default_value = ".",
            value_parser
        )]
        path: PathBuf,
    },

    #[clap(subcommand)]
    Link(Link),

    #[clap(subcommand)]
    File(FileCommand),

    #[clap(subcommand)]
    Storage(StorageCommand),
}

impl Command {
    /// Get the entry output format
    /// Default to Id
    pub fn entry(&self) -> Result<EntryOutput> {
        match self {
            Command::List {
                entry_id,
                entry_path,
                entry_link,
                ..
            } => {
                // Link can only be used alone
                if *entry_link {
                    if *entry_id || *entry_path {
                        return Err(AppError::InvalidEntryOption)?;
                    } else {
                        return Ok(EntryOutput::Link);
                    }
                }

                if *entry_id && *entry_path {
                    Ok(EntryOutput::Both)
                } else if *entry_path {
                    Ok(EntryOutput::Path)
                } else {
                    // Default to id
                    Ok(EntryOutput::Id)
                }
            }
            _ => Ok(EntryOutput::Id),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum StorageCommand {
    List {
        #[clap(parse(from_os_str))]
        root_dir: Option<PathBuf>,

        storage: Option<String>,

        #[clap(short, long)]
        versions: Option<bool>,

        #[clap(short, long, value_enum)]
        type_: Option<StorageType>,
    },
}

#[derive(Subcommand, Debug)]
pub enum FileCommand {
    Append {
        #[clap(parse(from_os_str))]
        root_dir: PathBuf,

        storage: String,

        id: String,

        content: String,

        #[clap(short, long, value_enum)]
        format: Option<Format>,

        #[clap(short, long, value_enum)]
        type_: Option<StorageType>,
    },

    Insert {
        #[clap(parse(from_os_str))]
        root_dir: PathBuf,

        storage: String,

        id: String,

        content: String,

        #[clap(short, long, value_enum)]
        format: Option<Format>,

        #[clap(short, long, value_enum)]
        type_: Option<StorageType>,
    },

    Read {
        #[clap(parse(from_os_str))]
        root_dir: PathBuf,

        storage: String,

        id: String,

        #[clap(short, long, value_enum)]
        type_: Option<StorageType>,
    },
}

#[derive(Subcommand, Debug)]
pub enum Link {
    Create {
        #[clap(parse(from_os_str))]
        root_dir: Option<PathBuf>,

        url: Option<String>,
        title: Option<String>,
        desc: Option<String>,
    },

    Load {
        #[clap(parse(from_os_str))]
        root_dir: Option<PathBuf>,

        #[clap(parse(from_os_str))]
        file_path: Option<PathBuf>,

        id: Option<ResourceId>,
    },
}
