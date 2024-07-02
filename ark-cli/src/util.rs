use crate::ResourceId;
use fs_index::index::ResourceIndex;
use fs_metadata::METADATA_STORAGE_FOLDER;
use fs_properties::PROPERTIES_STORAGE_FOLDER;
use fs_storage::{
    ARK_FOLDER, PREVIEWS_STORAGE_FOLDER, SCORE_STORAGE_FILE, STATS_FOLDER,
    TAG_STORAGE_FILE, THUMBNAILS_STORAGE_FOLDER,
};
use std::{
    env::current_dir,
    fs::{canonicalize, metadata, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    error::AppError,
    models::storage::{Storage, StorageType},
    ARK_CONFIG,
};

pub fn discover_roots(
    roots_cfg: &Option<PathBuf>,
) -> Result<Vec<PathBuf>, AppError> {
    if let Some(path) = roots_cfg {
        println!(
            "\tRoots config provided explicitly:\n\t\t{}",
            path.display()
        );
        let config = File::open(path)?;

        Ok(parse_roots(config))
    } else if let Ok(config) = File::open(ARK_CONFIG) {
        println!(
            "\tRoots config was found automatically:\n\t\t{}",
            &ARK_CONFIG
        );

        Ok(parse_roots(config))
    } else {
        println!("\tRoots config wasn't found.");

        println!("Looking for a folder containing tag storage:");
        let path =
            canonicalize(current_dir().expect("Can't open current directory!"))
                .expect("Couldn't canonicalize working directory!");

        let result = path.ancestors().find(|path| {
            println!("\t{}", path.display());
            storages_exists(path)
        });

        if let Some(root) = result {
            println!("Root folder found:\n\t{}", root.display());
            Ok(vec![root.to_path_buf()])
        } else {
            println!("Root folder wasn't found.");
            Ok(vec![])
        }
    }
}

pub fn provide_root(root_dir: &Option<PathBuf>) -> Result<PathBuf, AppError> {
    if let Some(path) = root_dir {
        Ok(path.clone())
    } else {
        Ok(current_dir()?)
    }
}

// Read-only structure
pub fn provide_index(root_dir: &PathBuf) -> ResourceIndex<ResourceId> {
    let rwlock =
        crate::provide_index(root_dir).expect("Failed to retrieve index");
    let index = &*rwlock.read().expect("Failed to lock index");
    index.clone()
}

pub fn monitor_index(
    root_dir: &Option<PathBuf>,
    interval: Option<u64>,
) -> Result<(), AppError> {
    let dir_path = provide_root(root_dir)?;

    println!("Building index of folder {}", dir_path.display());
    let start = Instant::now();

    let result = crate::provide_index(dir_path);
    let duration = start.elapsed();

    match result {
        Ok(rwlock) => {
            println!("Build succeeded in {:?}\n", duration);

            if let Some(millis) = interval {
                let mut index = rwlock.write().map_err(|_| {
                    AppError::StorageCreationError(
                        "Failed to write lock index".to_owned(),
                    )
                })?;
                loop {
                    let pause = Duration::from_millis(millis);
                    thread::sleep(pause);

                    let start = Instant::now();
                    match index.update_all() {
                        Err(msg) => println!("Oops! {}", msg),
                        Ok(diff) => {
                            index.store().expect("Could not store index");
                            let duration = start.elapsed();
                            println!("Updating succeeded in {:?}\n", duration);

                            if !diff.removed().is_empty() {
                                println!("Deleted: {:?}", diff.removed());
                            }
                            if !diff.added().is_empty() {
                                println!("Added: {:?}", diff.added());
                            }
                            if !diff.modified().is_empty() {
                                println!("Modified: {:?}", diff.modified());
                            }
                        }
                    }
                }
            } else {
                let index = rwlock.read().map_err(|_| {
                    AppError::StorageCreationError(
                        "Failed to read lock index".to_owned(),
                    )
                })?;

                println!("Here are {} entries in the index", index.len());

                for (key, resources) in index.collisions().iter() {
                    println!(
                        "Id {:?} calculated {} times",
                        key,
                        resources.len()
                    );
                }
            }
        }
        Err(err) => println!("Failure: {:?}", err),
    }

    Ok(())
}

pub fn storages_exists(path: &Path) -> bool {
    let meta = metadata(path.join(ARK_FOLDER));
    if let Ok(meta) = meta {
        return meta.is_dir();
    }

    false
}

pub fn parse_roots(config: File) -> Vec<PathBuf> {
    BufReader::new(config)
        .lines()
        .filter_map(|line| match line {
            Ok(path) => Some(PathBuf::from(path)),
            Err(msg) => {
                println!("{:?}", msg);
                None
            }
        })
        .collect()
}

pub fn timestamp() -> Duration {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards!")
}

pub fn translate_storage(
    root: &Option<PathBuf>,
    storage: &str,
) -> Option<(PathBuf, Option<StorageType>)> {
    if let Ok(path) = PathBuf::from_str(storage) {
        if path.exists() && path.is_dir() {
            return Some((path, None));
        }
    }

    match storage.to_lowercase().as_str() {
        "tags" => Some((
            provide_root(root)
                .ok()?
                .join(ARK_FOLDER)
                .join(TAG_STORAGE_FILE),
            Some(StorageType::File),
        )),
        "scores" => Some((
            provide_root(root)
                .ok()?
                .join(ARK_FOLDER)
                .join(SCORE_STORAGE_FILE),
            Some(StorageType::File),
        )),
        "stats" => Some((
            provide_root(root)
                .ok()?
                .join(ARK_FOLDER)
                .join(STATS_FOLDER),
            Some(StorageType::Folder),
        )),
        "properties" => Some((
            provide_root(root)
                .ok()?
                .join(ARK_FOLDER)
                .join(PROPERTIES_STORAGE_FOLDER),
            Some(StorageType::Folder),
        )),
        "metadata" => Some((
            provide_root(root)
                .ok()?
                .join(ARK_FOLDER)
                .join(METADATA_STORAGE_FOLDER),
            Some(StorageType::Folder),
        )),
        "previews" => Some((
            provide_root(root)
                .ok()?
                .join(ARK_FOLDER)
                .join(PREVIEWS_STORAGE_FOLDER),
            Some(StorageType::Folder),
        )),
        "thumbnails" => Some((
            provide_root(root)
                .ok()?
                .join(ARK_FOLDER)
                .join(THUMBNAILS_STORAGE_FOLDER),
            Some(StorageType::Folder),
        )),
        _ => None,
    }
}

pub fn read_storage_value(
    root_dir: &PathBuf,
    storage: &str,
    id: &str,
    type_: &Option<String>,
) -> Result<String, AppError> {
    let (file_path, storage_type) =
        translate_storage(&Some(root_dir.to_owned()), storage)
            .ok_or(AppError::StorageNotFound(storage.to_owned()))?;

    let storage_type = storage_type.unwrap_or(match type_ {
        Some(type_) => match type_.to_lowercase().as_str() {
            "file" => StorageType::File,
            "folder" => StorageType::Folder,
            _ => panic!("unknown storage type"),
        },
        None => StorageType::File,
    });

    let mut storage = Storage::new(file_path, storage_type)?;

    let resource_id =
        ResourceId::from_str(id).map_err(|_| AppError::InvalidEntryOption)?;
    storage.read(resource_id)
}
