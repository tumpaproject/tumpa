//! Tauri IPC commands. The frontend doesn't call these directly — the
//! desktop flows have no smartcard commands here; mobile flows route
//! through `libtumpa::card::mobile::MobileCardBackend`, which uses the
//! Rust-side `TumpaCard<R>` handle rather than going through IPC. We
//! still expose these so callers that want a pure-JS / debug flow can
//! drive the session directly. The keyring commands (save/read/clear
//! secret) ARE called from the frontend so the UI can cache card PINs
//! and on-disk key passphrases.

use tauri::{AppHandle, Runtime, State};

use crate::error::Result;
use crate::mobile::TumpaCard;
use crate::models::{
    BeginSessionRequest, BeginSessionResponse, ClearSecretRequest, EndSessionRequest,
    ReadSecretRequest, ReadSecretResponse, SaveSecretRequest, TransmitApduRequest,
    TransmitApduResponse,
};

#[tauri::command]
pub(crate) fn begin_session<R: Runtime>(
    _app: AppHandle<R>,
    plugin: State<'_, TumpaCard<R>>,
    req: BeginSessionRequest,
) -> Result<BeginSessionResponse> {
    plugin.begin_session(req)
}

#[tauri::command]
pub(crate) fn transmit_apdu<R: Runtime>(
    _app: AppHandle<R>,
    plugin: State<'_, TumpaCard<R>>,
    req: TransmitApduRequest,
) -> Result<TransmitApduResponse> {
    plugin.transmit_apdu(req)
}

#[tauri::command]
pub(crate) fn end_session<R: Runtime>(
    _app: AppHandle<R>,
    plugin: State<'_, TumpaCard<R>>,
    req: EndSessionRequest,
) -> Result<()> {
    plugin.end_session(req);
    Ok(())
}

#[tauri::command]
pub(crate) fn save_secret<R: Runtime>(
    _app: AppHandle<R>,
    plugin: State<'_, TumpaCard<R>>,
    req: SaveSecretRequest,
) -> Result<()> {
    plugin.save_secret(req)
}

#[tauri::command]
pub(crate) fn read_secret<R: Runtime>(
    _app: AppHandle<R>,
    plugin: State<'_, TumpaCard<R>>,
    req: ReadSecretRequest,
) -> Result<ReadSecretResponse> {
    plugin.read_secret(req)
}

#[tauri::command]
pub(crate) fn clear_secret<R: Runtime>(
    _app: AppHandle<R>,
    plugin: State<'_, TumpaCard<R>>,
    req: ClearSecretRequest,
) -> Result<()> {
    plugin.clear_secret(req)
}

#[tauri::command]
pub(crate) fn clear_all_secrets<R: Runtime>(
    _app: AppHandle<R>,
    plugin: State<'_, TumpaCard<R>>,
) -> Result<()> {
    plugin.clear_all_secrets()
}
