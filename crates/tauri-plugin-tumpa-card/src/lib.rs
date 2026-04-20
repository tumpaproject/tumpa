//! Tauri mobile plugin that bridges OpenPGP smartcard APDU I/O to
//! native NFC and USB APIs on Android and iOS.
//!
//! # Scope
//!
//! This plugin is **mobile-only**. Desktop builds use
//! [`wecanencrypt`]'s `card-pcsc` feature directly. Mobile builds
//! enable `card-external` + libtumpa's `card-mobile` feature, and
//! register this plugin to supply the APDU transport.
//!
//! # Vendor-neutral
//!
//! The plugin speaks generic ISO 7816-4 APDUs — no YubiKey-specific
//! code paths. Any card that answers `SELECT AID
//! D2760001240103040000000000000000` works: YubiKey, Nitrokey 3 /
//! Pro, Gnuk, on-device virtual cards.
//!
//! Native implementations:
//!
//! - **Android**: `IsoDep` for NFC (framework class, API 10+), `UsbManager`
//!   + CCID for USB-C. Any compliant NFC smartcard / USB CCID reader
//!   is handled uniformly.
//! - **iOS**: `NFCTagReaderSession` + `NFCISO7816Tag` for NFC; `TKSmartCard`
//!   via CryptoTokenKit for USB-C (pending Apple entitlement for
//!   third-party app distribution).
//!
//! # Usage
//!
//! ```no_run
//! # use tauri::Manager;
//! # fn example<R: tauri::Runtime>(app: &tauri::App<R>) {
//! app.handle().plugin(tauri_plugin_tumpa_card::init()).unwrap();
//! # }
//! ```
//!
//! Then from Rust — or from JavaScript via the IPC commands —
//! call `begin_session`, `transmit_apdu`, and `end_session`.

#![doc(html_no_source)]

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use error::{Error, Result};
pub use models::*;

mod commands;
mod error;
mod mobile;
mod models;

pub use mobile::{init as mobile_init, TumpaCard};

/// Accessor trait on [`Manager`] to fetch the plugin handle.
pub trait TumpaCardExt<R: Runtime> {
    fn tumpa_card(&self) -> &TumpaCard<R>;
}

impl<R: Runtime, T: Manager<R>> TumpaCardExt<R> for T {
    fn tumpa_card(&self) -> &TumpaCard<R> {
        self.state::<TumpaCard<R>>().inner()
    }
}

/// Construct the plugin. Install on the Tauri app builder:
///
/// ```no_run
/// # use tauri::Builder;
/// # let _ = Builder::<tauri::Wry>::default()
///     .plugin(tauri_plugin_tumpa_card::init())
/// # ;
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("tumpa-card")
        .invoke_handler(tauri::generate_handler![
            commands::begin_session,
            commands::transmit_apdu,
            commands::end_session,
            commands::save_secret,
            commands::read_secret,
            commands::clear_secret,
            commands::clear_all_secrets,
        ])
        .setup(|app, api| {
            let handle = mobile_init(app, api)?;
            app.manage(handle);
            Ok(())
        })
        .build()
}
