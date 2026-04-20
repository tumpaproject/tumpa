pub mod keystore;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod card;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod keyserver;

pub use keystore::*;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use card::*;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use keyserver::*;

use std::path::PathBuf;
use std::sync::Mutex;
use wecanencrypt::KeyStore;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::collections::HashMap;

/// Application state shared across Tauri commands
pub struct AppState {
    /// SQLite-backed keystore for persistent key storage
    pub keystore: Mutex<KeyStore>,
    /// Card-key associations: key fingerprint -> list of card idents
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub card_links: Mutex<HashMap<String, Vec<String>>>,
    /// Path to the card_links.json file
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub card_links_path: PathBuf,
    /// Base app data directory (used for keystore + any other persisted state)
    #[allow(dead_code)]
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(db_path: &str, data_dir: &std::path::Path) -> Self {
        let store = KeyStore::open(db_path).expect("Failed to open keystore");

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let card_links_path = data_dir.join("card_links.json");
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let card_links = load_card_links(&card_links_path);

        Self {
            keystore: Mutex::new(store),
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            card_links: Mutex::new(card_links),
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            card_links_path,
            data_dir: data_dir.to_path_buf(),
        }
    }

    /// Save card links to disk (desktop only)
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub fn save_card_links(&self) -> Result<(), String> {
        let links = self.card_links.lock().map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&*links)
            .map_err(|e| format!("Failed to serialize card links: {}", e))?;
        std::fs::write(&self.card_links_path, json)
            .map_err(|e| format!("Failed to save card links: {}", e))?;
        Ok(())
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn load_card_links(path: &std::path::Path) -> HashMap<String, Vec<String>> {
    if path.exists() {
        let data = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}
