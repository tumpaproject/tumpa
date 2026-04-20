//! Mobile-only: wire the `tauri-plugin-tumpa-card` IPC surface into
//! `wecanencrypt::card::external` so that the existing
//! `libtumpa::card::*` operations (`get_card_details`, `verify_user_pin`,
//! `upload_key_to_card`, …) work on Android / iOS the same way they
//! work on desktop.
//!
//! The flow for any card op on mobile:
//!
//! 1. libtumpa calls `wecanencrypt::card::get_card_backend(None)`.
//! 2. That dispatches to the registered external provider (see below).
//! 3. The provider builds a `TauriCardBridge` backed by the mobile
//!    plugin, wraps it in `libtumpa::card::mobile::MobileCardBackend`,
//!    and hands it back as a `Box<dyn CardBackend + Send + Sync>`.
//! 4. `MobileCardBackend::new` calls `begin_session` synchronously
//!    — this triggers the NFC prompt / USB permission dialog on the
//!    native side and blocks until the user taps / plugs in a card.
//! 5. APDUs flow through `transmit_apdu` for the duration of the op.
//! 6. When the `Card` / `OpenPGP` handle inside wecanencrypt drops,
//!    so does the backend, and `end_session` runs in `Drop`.

use std::sync::Mutex;

use card_backend::{CardBackend, SmartcardError};
use libtumpa::card::{
    external,
    mobile::{CardBridge, MobileCardBackend},
    WecanencryptCardError, WecanencryptError,
};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_tumpa_card::{
    BeginSessionRequest, EndSessionRequest, TransmitApduRequest, Transport, TumpaCardExt,
};

/// Shared UI-driven transport preference. Exposed to JS via
/// `set_card_transport` / `get_card_transport` so the user can pick
/// NFC or USB explicitly when both are plausible (e.g. Android with a
/// YubiKey both taped and plugged in), or fall back to `Auto` and let
/// the native plugin decide.
pub struct CardTransportState(pub Mutex<Transport>);

impl Default for CardTransportState {
    fn default() -> Self {
        Self(Mutex::new(Transport::Auto))
    }
}

#[tauri::command]
pub fn set_card_transport<R: Runtime>(
    app: AppHandle<R>,
    transport: String,
) -> Result<(), String> {
    let t = match transport.as_str() {
        "nfc" => Transport::Nfc,
        "usb" => Transport::Usb,
        "auto" => Transport::Auto,
        other => return Err(format!("unknown transport: {other}")),
    };
    let state = app.state::<CardTransportState>();
    *state.0.lock().unwrap() = t;
    Ok(())
}

/// Short-form OpenPGP application AID (RID `D276000124` + PIX `01`).
///
/// ISO 7816-4 SELECT-by-name (P1=0x04) will match an application whose
/// full AID starts with this prefix, so sending 6 bytes is the
/// portable way to reach the OpenPGP applet. The full 16-byte AID
/// includes version, manufacturer, and serial; hard-coding it (e.g.
/// `…030400000000…`) only matches a card that happens to have those
/// exact version/manufacturer bytes, which caused a `6A82 file not
/// found` from YubiKey NFC cards whose real AID is e.g.
/// `D276000124010304 0006 XXXXXXXX 0000`.
///
/// (iOS Info.plist `iso7816.select-identifiers` uses the full 16-byte
/// form as a pattern; that's a separate concern — CoreNFC does the
/// matching, not us.)
const OPENPGP_AID: &[u8] = &[0xd2, 0x76, 0x00, 0x01, 0x24, 0x01];

/// Bridge that routes `MobileCardBackend`'s synchronous `CardBridge`
/// methods to the Tauri mobile plugin's async native layer.
struct TauriCardBridge<R: Runtime> {
    app: AppHandle<R>,
    // The native plugin identifies each session with a UUID-ish string.
    // We keep it here so `transmit_apdu` and `end_session` can route
    // to the right card.
    session_id: Mutex<Option<String>>,
    // Transport chosen by the UI before the backend was constructed.
    // Phase 1 hardcodes NFC; future work: make the UI ask the user.
    transport: Transport,
}

impl<R: Runtime> TauriCardBridge<R> {
    fn new(app: AppHandle<R>, transport: Transport) -> Self {
        Self {
            app,
            session_id: Mutex::new(None),
            transport,
        }
    }
}

impl<R: Runtime> CardBridge for TauriCardBridge<R> {
    fn begin_session(&self) -> Result<(), SmartcardError> {
        let plugin = self.app.tumpa_card();
        let resp = plugin
            .begin_session(BeginSessionRequest {
                transport: self.transport,
                applet_aid: OPENPGP_AID.to_vec(),
            })
            .map_err(|e| SmartcardError::Error(format!("begin_session: {e}")))?;
        *self.session_id.lock().unwrap() = Some(resp.session_id);
        Ok(())
    }

    fn transmit_apdu(&self, cmd: &[u8]) -> Result<Vec<u8>, SmartcardError> {
        let session_id = self
            .session_id
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| SmartcardError::Error("no-active-session".into()))?;
        let plugin = self.app.tumpa_card();
        let resp = plugin
            .transmit_apdu(TransmitApduRequest {
                session_id,
                apdu: cmd.to_vec(),
            })
            .map_err(|e| SmartcardError::Error(format!("transmit_apdu: {e}")))?;
        Ok(resp.response)
    }

    fn end_session(&self) {
        let session_id = match self.session_id.lock().unwrap().take() {
            Some(id) => id,
            None => return,
        };
        let plugin = self.app.tumpa_card();
        plugin.end_session(EndSessionRequest { session_id });
    }
}

/// Register the mobile card transport with wecanencrypt.
///
/// Idempotent — subsequent calls are no-ops because
/// `wecanencrypt::card::external::set_backend_provider` uses a
/// `OnceLock`.
pub(crate) fn register_backend_provider<R: Runtime + 'static>(
    app: AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    external::set_backend_provider(move |_ident: Option<&str>| {
        // Read the UI-driven transport choice each time we mint a
        // backend — the user may switch between NFC and USB between
        // card operations without restarting the app. Default is
        // `Auto`, which lets the native plugin pick (USB if plugged
        // in, NFC otherwise).
        let transport = app
            .state::<CardTransportState>()
            .0
            .lock()
            .map(|t| *t)
            .unwrap_or(Transport::Auto);
        let bridge = TauriCardBridge::new(app.clone(), transport);
        let backend = MobileCardBackend::new(bridge).map_err(|e| {
            WecanencryptError::Card(WecanencryptCardError::CommunicationError(e.to_string()))
        })?;
        Ok(Box::new(backend) as Box<dyn CardBackend + Send + Sync>)
    })
    .ok(); // Already-registered is fine.

    Ok(())
}
