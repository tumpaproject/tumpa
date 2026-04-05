pub mod keystore;
pub mod card;

pub use keystore::*;
pub use card::*;

use std::sync::Mutex;
use wecanencrypt::KeyStore;

/// Application state shared across Tauri commands
pub struct AppState {
    /// SQLite-backed keystore for persistent key storage
    pub keystore: Mutex<KeyStore>,
}

impl AppState {
    pub fn new(db_path: &str) -> Self {
        let store = KeyStore::open(db_path)
            .expect("Failed to open keystore");
        Self {
            keystore: Mutex::new(store),
        }
    }
}
