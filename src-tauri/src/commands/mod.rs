pub mod keystore;

// The card commands module is available on desktop and mobile. On
// mobile the PCSC-only bits (enumeration + card-link auto-detect) are
// gated off inside the module; what remains routes through
// `wecanencrypt::card::external` → `MobileCardBackend` → the
// tauri-plugin-tumpa-card native bridge.
pub mod card;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod keyserver;

pub use keystore::*;
pub use card::*;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use keyserver::*;

use std::path::PathBuf;
use std::sync::Mutex;
use libtumpa::KeyStore;

/// Application state shared across Tauri commands.
///
/// Card↔key links used to live in `~/.tumpa/card_links.json` with an
/// in-memory `HashMap` mirror in this struct. libtumpa now persists the
/// same information in the keystore's `card_keys` table, so this struct
/// no longer carries that state.
pub struct AppState {
    pub keystore: Mutex<KeyStore>,
    /// Base app data directory. Kept around in case future commands need
    /// to write auxiliary files next to `keys.db`.
    #[allow(dead_code)]
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(db_path: &str, data_dir: &std::path::Path) -> Self {
        let store = KeyStore::open(db_path).expect("Failed to open keystore");
        Self {
            keystore: Mutex::new(store),
            data_dir: data_dir.to_path_buf(),
        }
    }
}
