//! App-mode commands: swap the in-process `KeyStore` between the
//! on-disk store (`Persistent`) and an ephemeral in-memory SQLite
//! store (`OneShot`).
//!
//! Callers: the sidebar / mobile overflow menu. Never triggered
//! automatically — always an explicit user action.
//!
//! Safety properties we rely on:
//!
//! - `wecanencrypt::KeyStore::open_in_memory()` returns a store whose
//!   schema matches the disk store byte-for-byte, so every existing
//!   command (list, import, generate, upload, export, …) works unchanged
//!   while the Mutex holds the in-memory variant.
//! - Dropping the old `KeyStore` drops its `rusqlite::Connection`; for
//!   the One Shot path that was the in-memory SQLite db, so the
//!   secret-key rows it held are released back to the allocator. The
//!   OS reclaims the pages when the process exits. We deliberately
//!   don't touch the on-disk file across `enter_one_shot` —
//!   persistent data is left untouched on disk, just hidden from the
//!   UI while One Shot is active.
//! - `exit_one_shot` doesn't re-open the disk store in-process —
//!   instead it drops the in-memory store and calls `app.exit(0)`.
//!   A fresh launch gives the cleanest "nothing leaked" guarantee:
//!   the heap, webview buffers, any async futures that were holding a
//!   reference to the ephemeral store — all go with the process.

use libtumpa::KeyStore;
use tauri::{AppHandle, Emitter, Runtime, State};

use super::{AppState, KeyStoreMode};

/// Event name broadcast when the mode changes. The UI subscribes to
/// this to refresh the key list and toggle the banner.
const MODE_EVENT: &str = "tumpa:mode-changed";

fn mode_str(m: KeyStoreMode) -> &'static str {
    match m {
        KeyStoreMode::Persistent => "persistent",
        KeyStoreMode::OneShot => "one-shot",
    }
}

/// Query the current mode. Used by the UI on startup to initialise the
/// banner / menu state.
#[tauri::command]
pub fn get_app_mode(state: State<'_, AppState>) -> Result<String, String> {
    let m = *state.mode.lock().map_err(|e| e.to_string())?;
    Ok(mode_str(m).into())
}

/// Swap in a fresh in-memory keystore. Any keys currently in the
/// (previous) in-memory store are dropped. The on-disk store — if any —
/// is untouched; it'll reappear when the user calls `exit_one_shot`.
#[tauri::command]
pub fn enter_one_shot<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let fresh = KeyStore::open_in_memory().map_err(|e| format!("open_in_memory: {e}"))?;
    {
        let mut store = state.keystore.lock().map_err(|e| e.to_string())?;
        *store = fresh;
    }
    *state.mode.lock().map_err(|e| e.to_string())? = KeyStoreMode::OneShot;
    let _ = app.emit(MODE_EVENT, "one-shot");
    Ok(())
}

/// Exit One Shot by terminating the process. We deliberately do NOT
/// swap back to the disk keystore in-process — a fresh launch gives
/// the cleanest "nothing leaked" guarantee (heap pages, webview
/// buffers, anything the Vue side happened to hold onto all go away
/// with the process). Next launch boots in `Persistent` mode per
/// `AppState::new`.
///
/// We emit the mode event first so any UI listeners can react
/// synchronously, clear the in-memory store, then call `app.exit(0)`.
#[tauri::command]
pub fn exit_one_shot<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Drop the in-memory store deterministically before the process
    // teardown kicks in. Opening a throwaway store lets us release the
    // rusqlite::Connection that held the ephemeral key rows.
    {
        let mut store = state.keystore.lock().map_err(|e| e.to_string())?;
        *store = KeyStore::open_in_memory().map_err(|e| format!("swap-out: {e}"))?;
    }
    *state.mode.lock().map_err(|e| e.to_string())? = KeyStoreMode::Persistent;
    let _ = app.emit(MODE_EVENT, "persistent");
    app.exit(0);
    Ok(())
}
