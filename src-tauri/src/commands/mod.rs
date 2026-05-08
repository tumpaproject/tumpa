pub mod keystore;

// The card commands module is available on desktop and mobile. On
// mobile the PCSC-only bits (enumeration + card-link auto-detect) are
// gated off inside the module; what remains routes through
// `wecanencrypt::card::external` → `MobileCardBackend` → the
// tauri-plugin-tumpa-card native bridge.
pub mod card;

pub mod mode;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod keyserver;

pub use keystore::*;
pub use card::*;
pub use mode::*;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use keyserver::*;

use std::path::PathBuf;
use std::sync::Mutex;
use libtumpa::KeyStore;

/// Which flavour of keystore the app is currently running against.
///
/// Swapped at runtime by the One Shot commands (see `mode.rs`) — every
/// command that takes `State<'_, AppState>` reads `state.keystore`
/// without caring which variant is underneath.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStoreMode {
    /// Default. Backed by `keys.db` on disk under `data_dir`.
    Persistent,
    /// In-memory SQLite — nothing written to disk, keys are gone when
    /// the app closes.
    OneShot,
}

/// Application state shared across Tauri commands.
///
/// `keystore` stays a `Mutex<KeyStore>` so every existing command that
/// takes `State<'_, AppState>` keeps working unchanged; `enter_one_shot`
/// swaps the value inside the Mutex to an in-memory store. Exiting
/// One Shot is handled by terminating the process, so there's no
/// need to remember the disk path here.
pub struct AppState {
    pub keystore: Mutex<KeyStore>,
    pub mode: Mutex<KeyStoreMode>,
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
            mode: Mutex::new(KeyStoreMode::Persistent),
            data_dir: data_dir.to_path_buf(),
        }
    }
}
