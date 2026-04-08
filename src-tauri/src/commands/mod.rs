pub mod keystore;
pub mod card;
pub mod keyserver;

pub use keystore::*;
pub use card::*;
pub use keyserver::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use wecanencrypt::KeyStore;

/// Application state shared across Tauri commands
pub struct AppState {
    /// SQLite-backed keystore for persistent key storage
    pub keystore: Mutex<KeyStore>,
    /// Card-key associations: key fingerprint -> list of card idents
    pub card_links: Mutex<HashMap<String, Vec<String>>>,
    /// Path to the card_links.json file
    pub card_links_path: PathBuf,
}

impl AppState {
    pub fn new(db_path: &str, tumpa_dir: &std::path::Path) -> Self {
        let store = KeyStore::open(db_path)
            .expect("Failed to open keystore");

        let card_links_path = tumpa_dir.join("card_links.json");
        let card_links = load_card_links(&card_links_path);

        Self {
            keystore: Mutex::new(store),
            card_links: Mutex::new(card_links),
            card_links_path,
        }
    }

    /// Save card links to disk
    pub fn save_card_links(&self) -> Result<(), String> {
        let links = self.card_links.lock().map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&*links)
            .map_err(|e| format!("Failed to serialize card links: {}", e))?;
        std::fs::write(&self.card_links_path, json)
            .map_err(|e| format!("Failed to save card links: {}", e))?;
        Ok(())
    }
}

fn load_card_links(path: &std::path::Path) -> HashMap<String, Vec<String>> {
    if path.exists() {
        let data = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}
