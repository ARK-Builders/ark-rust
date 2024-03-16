use std::path::PathBuf;

use arklib::id::ResourceId;
use clap::{Parser, Subcommand};

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

        #[clap(long)]
        entry: Option<EntryOutput>,

        #[clap(long, short = 'i', action)]
        entry_id: bool,

        #[clap(long, short = 'p', action)]
        entry_path: bool,

        #[clap(long, short = 'l', action)]
        entry_link: bool,

        #[clap(long, short, action)]
        modified: bool,

        #[clap(long, short, action)]
        tags: bool,

        #[clap(long, short, action)]
        scores: bool,

        #[clap(long)]
        sort: Option<Sort>,

        #[clap(long)]
        filter: Option<String>,
    },

    #[clap(subcommand)]
    Link(Link),

    #[clap(subcommand)]
    File(FileCommand),

    #[clap(subcommand)]
    Storage(StorageCommand),
}

#[derive(Subcommand, Debug)]
pub enum StorageCommand {
    List {
        #[clap(parse(from_os_str))]
        root_dir: Option<PathBuf>,

        storage: Option<String>,

        #[clap(short, long)]
        versions: Option<bool>,

        #[clap(short, long)]
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

        #[clap(short, long)]
        format: Option<Format>,

        #[clap(short, long)]
        type_: Option<StorageType>,
    },

    Insert {
        #[clap(parse(from_os_str))]
        root_dir: PathBuf,

        storage: String,

        id: String,

        content: String,

        #[clap(short, long)]
        format: Option<Format>,

        #[clap(short, long)]
        type_: Option<StorageType>,
    },

    Read {
        #[clap(parse(from_os_str))]
        root_dir: PathBuf,

        storage: String,

        id: String,

        #[clap(short, long)]
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
