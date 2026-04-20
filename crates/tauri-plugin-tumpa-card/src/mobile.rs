//! Platform bridge — turns `PluginApi<R, C>` into a typed `TumpaCard<R>`
//! handle that the host app can call from Rust. Delegates each command
//! to the corresponding Kotlin / Swift method via
//! `PluginHandle::run_mobile_plugin` (synchronous, blocking — safe from
//! worker threads, not from the async runtime's reactor thread).

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::error::{Error, Result};
use crate::models::{
    BeginSessionRequest, BeginSessionResponse, ClearSecretRequest, EndSessionRequest,
    ReadSecretRequest, ReadSecretResponse, SaveSecretRequest, TransmitApduRequest,
    TransmitApduResponse,
};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "in.kushaldas.tumpa.card";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_tumpa_card);

/// Register the Android / iOS plugin classes and build the typed handle
/// the host app uses to call them.
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<TumpaCard<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "TumpaCardPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_tumpa_card)?;

    // Only Android and iOS register a handle. The `DesktopUnsupported`
    // error arm in `error.rs` exists for builds that accidentally
    // include the plugin on a non-mobile target.
    #[cfg(not(mobile))]
    return Err(Error::DesktopUnsupported);

    #[cfg(mobile)]
    Ok(TumpaCard(handle))
}

/// Typed handle to the native card plugin. Cheap to clone.
#[derive(Debug)]
pub struct TumpaCard<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Clone for TumpaCard<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: Runtime> TumpaCard<R> {
    /// Begin a new smartcard session over the chosen transport. Blocks
    /// until the card is present and the OpenPGP applet has been
    /// selected, or returns an error (user cancelled, transport
    /// failure, applet rejected the SELECT).
    pub fn begin_session(&self, req: BeginSessionRequest) -> Result<BeginSessionResponse> {
        self.0
            .run_mobile_plugin::<BeginSessionResponse>("beginSession", req)
            .map_err(map_native_err)
    }

    /// Send one APDU and return the response bytes (status word
    /// included — the caller inspects `resp[resp.len()-2..]` per usual
    /// ISO 7816-4 convention).
    pub fn transmit_apdu(&self, req: TransmitApduRequest) -> Result<TransmitApduResponse> {
        self.0
            .run_mobile_plugin::<TransmitApduResponse>("transmitApdu", req)
            .map_err(map_native_err)
    }

    /// End the session, releasing NFC / closing the CCID pipe. Best
    /// effort — errors are logged but don't propagate, so the caller
    /// can invoke this from a `Drop` impl.
    pub fn end_session(&self, req: EndSessionRequest) {
        if let Err(e) = self
            .0
            .run_mobile_plugin::<serde_json::Value>("endSession", req)
        {
            log::debug!("end_session: native bridge returned error (best-effort): {e}");
        }
    }

    /// Save a secret to the platform keyring under the given opaque
    /// identifier. Requires the device to have a passcode configured.
    /// Does **not** prompt biometrics — only reads are gated.
    ///
    /// `key` is assigned by the caller; see [`SaveSecretRequest`] for
    /// the recommended naming scheme.
    pub fn save_secret(&self, req: SaveSecretRequest) -> Result<()> {
        self.0
            .run_mobile_plugin::<serde_json::Value>("saveSecret", req)
            .map(|_| ())
            .map_err(map_native_err)
    }

    /// Read a secret from the platform keyring. Always presents a
    /// biometric prompt; on cancel or lockout the call rejects. Never
    /// returns silently.
    pub fn read_secret(&self, req: ReadSecretRequest) -> Result<ReadSecretResponse> {
        self.0
            .run_mobile_plugin::<ReadSecretResponse>("readSecret", req)
            .map_err(map_native_err)
    }

    /// Remove a single secret entry. No biometric prompt — the user
    /// already authenticated to unlock the app, and wiping a cached
    /// secret is a defensive operation.
    pub fn clear_secret(&self, req: ClearSecretRequest) -> Result<()> {
        self.0
            .run_mobile_plugin::<serde_json::Value>("clearSecret", req)
            .map(|_| ())
            .map_err(map_native_err)
    }

    /// Remove every secret the plugin has stored for this app. Called
    /// from "Forget saved PINs / passphrases" actions and on explicit
    /// sign-out.
    pub fn clear_all_secrets(&self) -> Result<()> {
        self.0
            .run_mobile_plugin::<serde_json::Value>("clearAllSecrets", ())
            .map(|_| ())
            .map_err(map_native_err)
    }
}

fn map_native_err(e: tauri::plugin::mobile::PluginInvokeError) -> Error {
    // Tauri reshapes native errors into PluginInvokeError; we surface
    // them as Error::Native so the caller can tell them apart from
    // card-level failures. The native side uses reject() with a short
    // keyword ("cancelled" / "no-active-session" / "no-secret-saved"
    // / …) in the message to let us map common cases onto typed
    // variants.
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
