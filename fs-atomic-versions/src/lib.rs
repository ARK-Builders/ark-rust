use lazy_static::lazy_static;
use std::{
    path::PathBuf,
    sync::{Once, RwLock},
};

pub mod app_id;
pub mod atomic;

pub static INIT: Once = Once::new();

pub const APP_ID_FILE: &str = "app_id";

lazy_static! {
    pub static ref APP_ID_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);
}

pub fn initialize() {
    INIT.call_once(|| {
        log::info!("Initializing arklib");
        app_id::load("./").unwrap();
    });
}
