use std::{io::Write, path::PathBuf};

use crate::{
    create_dir_all, dir, discover_roots, home_dir, storages_exists, timestamp,
    AppError, CopyOptions, File, ARK_BACKUPS_PATH, ARK_FOLDER,
    ROOTS_CFG_FILENAME,
};

#[derive(Clone, Debug, clap::Args)]
#[clap(name = "backup", about = "Backup the ark managed folder")]
pub struct Backup {
    #[clap(value_parser, help = "Path to the root directory")]
    roots_cfg: Option<PathBuf>,
}

impl Backup {
    pub fn run(&self) -> Result<(), AppError> {
        let timestamp = timestamp().as_secs();
        let backup_dir = home_dir()
            .ok_or(AppError::HomeDirNotFound)?
            .join(ARK_BACKUPS_PATH)
            .join(timestamp.to_string());

        if backup_dir.is_dir() {
            println!("Wait at least 1 second, please!");
            std::process::exit(0)
        }

        println!("Preparing backup:");
        let roots = discover_roots(&self.roots_cfg)?;

        let (valid, invalid): (Vec<PathBuf>, Vec<PathBuf>) = roots
            .into_iter()
            .partition(|root| storages_exists(root));

        if !invalid.is_empty() {
            println!("These folders don't contain any storages:");
            invalid
                .into_iter()
                .for_each(|root| println!("\t{}", root.display()));
        }

        if valid.is_empty() {
            println!("Nothing to backup. Bye!");
            std::process::exit(0)
        }

        create_dir_all(&backup_dir).map_err(|_| {
            AppError::BackupCreationError(
                "Couldn't create backup directory!".to_owned(),
            )
        })?;

        let mut roots_cfg_backup =
            File::create(backup_dir.join(ROOTS_CFG_FILENAME))?;

        valid.iter().for_each(|root| {
            let res = writeln!(roots_cfg_backup, "{}", root.display());
            if let Err(e) = res {
                println!("Failed to write root to backup file: {}", e);
            }
        });

        println!("Performing backups:");
        valid
            .into_iter()
            .enumerate()
            .for_each(|(i, root)| {
                println!("\tRoot {}", root.display());
                let storage_backup = backup_dir.join(i.to_string());

                let mut options = CopyOptions::new();
                options.overwrite = true;
                options.copy_inside = true;

                let result =
                    dir::copy(root.join(ARK_FOLDER), storage_backup, &options);

                if let Err(e) = result {
                    println!("\t\tFailed to copy storages!\n\t\t{}", e);
                }
            });

        println!("Backup created:\n\t{}", backup_dir.display());

        Ok(())
    }
}
