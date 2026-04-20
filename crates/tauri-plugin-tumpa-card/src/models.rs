//! Shared request / response types sent between Rust, Kotlin, and Swift
//! over the Tauri mobile plugin IPC bridge. Field names are camelCase
//! on the wire so they match the JS / Kotlin / Swift idioms on the
//! native side.

use serde::{Deserialize, Serialize};

/// Which transport to use when beginning a card session.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    /// Contactless via Android `IsoDep` or iOS `NFCTagReaderSession` +
    /// `NFCISO7816Tag`. Shows the OS NFC prompt on iOS; on Android just
    /// waits for a tap after enabling foreground dispatch.
    Nfc,
    /// Wired via Android USB host (`UsbManager` + CCID) or iOS
    /// `TKSmartCard` via USB-C accessory. The OS permission dialog
    /// appears the first time an eligible reader is plugged in.
    Usb,
}

/// Argument to `begin_session`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginSessionRequest {
    pub transport: Transport,
    /// OpenPGP applet AID to select after establishing the session. The
    /// caller usually passes the standard OpenPGP card AID
    /// `D2760001240103040000000000000000`. The native side calls
    /// `SELECT` with this AID before returning; if the card rejects
    /// the SELECT, `begin_session` fails with `Error::Card`.
    pub applet_aid: Vec<u8>,
}

/// Response from `begin_session`. Small on purpose — the session itself
/// lives in the native plugin, identified by an opaque `session_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginSessionResponse {
    /// Opaque identifier for this session. The caller passes it back
    /// on subsequent `transmit_apdu` / `end_session` calls so the
    /// native side can route APDUs to the right card.
    pub session_id: String,
    /// ATR-like capability hint, if the native side can surface it
    /// (NFC historical bytes on ISO-DEP, ATR on CCID). `None` when the
    /// transport doesn't expose it.
    pub atr: Option<Vec<u8>>,
}

/// Argument to `transmit_apdu`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransmitApduRequest {
    pub session_id: String,
    pub apdu: Vec<u8>,
}

/// Response from `transmit_apdu`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransmitApduResponse {
    pub response: Vec<u8>,
}

/// Argument to `end_session`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndSessionRequest {
    pub session_id: String,
}

// -- Keyring-backed secret storage --------------------------------

/// The plugin exposes a generic keyring: any caller-supplied string
/// identifier maps to a biometric-gated secret (bytes). tumpa assigns
/// the identifiers — the plugin just stores what it's told.
///
/// Recommended identifier scheme used by tumpa:
///
/// - `card.pin.user`            — OpenPGP card user PIN (PW1)
/// - `card.pin.admin`           — OpenPGP card admin PIN (PW3)
/// - `key.pass.<FINGERPRINT>`   — passphrase for the on-disk secret
///                                key with the given uppercase
///                                fingerprint
///
/// Identifiers are opaque to the plugin. Picking a collision-free
/// scheme is the caller's responsibility; treat a duplicate save as
/// an overwrite.

/// Argument to `save_secret`. `secret` is carried as raw bytes since a
/// card PIN may legally contain any byte values and a passphrase is
/// UTF-8. UI callers encode strings with `str::as_bytes()` before
/// passing them in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSecretRequest {
    pub key: String,
    pub secret: Vec<u8>,
}

/// Argument to `read_secret`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadSecretRequest {
    pub key: String,
    /// User-facing message shown on the biometric prompt (Face ID /
    /// Touch ID / fingerprint). Short and action-oriented — the OS
    /// truncates anything longer than a couple of lines.
    pub reason: String,
}

/// Response from `read_secret`. Always biometric-gated on the native
/// side; if the user cancels or fails auth the plugin rejects with
/// `Error::Cancelled` rather than returning `Option::None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadSecretResponse {
    pub secret: Vec<u8>,
}

/// Argument to `clear_secret`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearSecretRequest {
    pub key: String,
}
