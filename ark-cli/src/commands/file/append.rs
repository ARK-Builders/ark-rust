use std::{path::PathBuf, str::FromStr};

use crate::{
    models::storage::{Storage, StorageType},
    translate_storage, AppError, Format, ResourceId,
};

use data_error::ArklibError;

#[derive(Clone, Debug, clap::Args)]
#[clap(name = "append", about = "Append content to a resource")]
pub struct Append {
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
    #[clap(help = "Content to append to the resource")]
    content: String,
    #[clap(
        short,
        long,
        value_enum,
        default_value = "raw",
        help = "Format of the resource"
    )]
    format: Option<Format>,
    #[clap(short, long, value_enum, help = "Storage kind of the resource")]
    kind: Option<StorageType>,
}

impl Append {
    pub fn run(&self) -> Result<(), AppError> {
        let (file_path, storage_type) =
            translate_storage(&Some(self.root_dir.to_owned()), &self.storage)
                .ok_or(AppError::StorageNotFound(self.storage.to_owned()))?;

        let storage_type = storage_type.unwrap_or(match self.kind {
            Some(t) => t,
            None => StorageType::File,
        });

        let format = self.format.unwrap();

        let mut storage = Storage::new(file_path, storage_type)?;

        let resource_id = ResourceId::from_str(&self.id)
            .map_err(|_e| AppError::ArklibError(ArklibError::Parse))?;

        storage.append(resource_id, &self.content, format)?;

        Ok(())
    }
}
