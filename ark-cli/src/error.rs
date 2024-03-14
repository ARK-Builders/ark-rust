use arklib::ArklibError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InlineJsonParseError {
    #[error("Invalid JSON: entries must be key-value pairs seperated by ':'")]
    InvalidKeyValPair,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Couldn't retrieve home directory!")]
    HomeDirNotFound,

    #[error("Couldn't create .ark directory: {0}")]
    ArkDirectoryCreationError(String),

    #[error("Couldn't load app id: {0}")]
    AppIdLoadError(String),

    #[error("Could not provide/read index: {0}")]
    IndexError(String),

    #[error("Could not create storage: {0}")]
    StorageCreationError(String),

    #[error("Failed to create link: {0}")]
    LinkCreationError(String),

    #[error("Could not load link: {0}")]
    LinkLoadError(String),

    #[error("File operation error: {0}")]
    FileOperationError(String),

    #[error("Failed to create backup: {0}")]
    BackupCreationError(String),

    #[error("Unknown render option")]
    InvalidRenderOption,

    #[error("Storage not found: {0}")]
    StorageNotFound(String),

    #[error("Invalid entry option")]
    InvalidEntryOption,

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    ArklibError(#[from] ArklibError),

    #[error(transparent)]
    InlineJsonParseError(#[from] InlineJsonParseError),
}
