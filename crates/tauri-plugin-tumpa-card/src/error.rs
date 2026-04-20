//! Plugin error type. The native side returns errors as strings over the
//! IPC bridge (`run_mobile_plugin` surfaces them as `serde_json::Value` on
//! failure); we reshape them into a small tagged enum so the caller can
//! distinguish "no card is currently available" from "the native bridge
//! broke" from "the card itself returned a non-success status word".

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No card session is active. `begin_session` needs to be called
    /// (or the user needs to tap / plug in a card) before operations
    /// can proceed.
    #[error("no active card session")]
    NoActiveSession,

    /// `read_secret` was called for an identifier that has never been
    /// saved, or that was cleared by a prior `clear_secret` /
    /// `clear_all_secrets`. The caller should fall back to prompting
    /// the user.
    #[error("no saved secret under that key")]
    NoSecretSaved,

    /// The user cancelled the NFC prompt / declined the USB permission
    /// dialog, or dismissed the biometric prompt during a
    /// `read_secret` call.
    #[error("cancelled by user")]
    Cancelled,

    /// Error returned by the Tauri mobile plugin bridge itself — JSON
    /// serialization, IPC channel, or a native panic.
    #[error("native bridge error: {0}")]
    Native(String),

    /// Error surfaced by the smartcard (non-`0x9000` SW, transport
    /// timeout, aborted transaction, etc.).
    #[error("card error: {0}")]
    Card(String),

    /// The plugin isn't available on this platform. Raised from
    /// desktop-targeted build configurations that accidentally link the
    /// plugin — keeps the error path obvious rather than panicking.
    #[cfg(not(mobile))]
    #[error("tauri-plugin-tumpa-card is only available on Android and iOS")]
    DesktopUnsupported,

    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Bridge errors surfaced by `register_android_plugin` / `register_ios_plugin`
// / `run_mobile_plugin` come back as PluginInvokeError — reshape into the
// typed Native variant so the caller can use `?` freely.
impl From<tauri::plugin::mobile::PluginInvokeError> for Error {
    fn from(e: tauri::plugin::mobile::PluginInvokeError) -> Self {
        let msg = e.to_string();
        if msg.contains("cancelled") {
            Error::Cancelled
        } else if msg.contains("no-active-session") {
            Error::NoActiveSession
        } else if msg.contains("no-secret-saved") {
            Error::NoSecretSaved
        } else {
            Error::Native(msg)
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
