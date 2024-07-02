use std::{path::PathBuf, str::FromStr};

use crate::{
    models::storage::{Storage, StorageType},
    translate_storage, AppError, ResourceId,
};

use data_error::ArklibError;

#[derive(Clone, Debug, clap::Args)]
#[clap(name = "read", about = "Read content from a resource")]
pub struct Read {
    #[clap(
        value_parser,
        default_value = ".",
        help = "Root directory of the ark managed folder"
    )]
    root_dir: PathBuf,
    #[clap(help = "Storage name")]
    storage: String,
    #[clap(help = "ID of the resource to append to")]
    id: String,
    #[clap(short, long, value_enum, help = "Storage kind of the resource")]
    kind: Option<StorageType>,
}

impl Read {
    pub fn run(&self) -> Result<(), AppError> {
        let (file_path, storage_type) =
            translate_storage(&Some(self.root_dir.to_owned()), &self.storage)
                .ok_or(AppError::StorageNotFound(self.storage.to_owned()))?;

        let storage_type = storage_type.unwrap_or(match self.kind {
            Some(t) => t,
            None => StorageType::File,
        });

        let mut storage = Storage::new(file_path, storage_type)?;

        let resource_id = ResourceId::from_str(&self.id)
            .map_err(|_e| AppError::ArklibError(ArklibError::Parse))?;

        let output = storage.read(resource_id)?;

        println!("{}", output);

        Ok(())
    }
}
