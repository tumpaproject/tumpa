//! Keyserver (keys.openpgp.org / VKS) `#[tauri::command]` functions.
//!
//! Exports the public key under the keystore lock, drops the lock, then
//! hands the armored string to `libtumpa::network::vks_upload_armored`.
//! The split keeps the async command future Send (rusqlite's KeyStore is
//! !Send, so we can't hold a guard across `.await`).

use libtumpa::{key, network};
use serde::Serialize;
use tauri::State;

use super::AppState;

#[derive(Serialize)]
pub struct KeyserverUploadResult {
    pub fingerprint: String,
    /// Per-email verification status: "unpublished", "published", "revoked", "pending".
    pub email_status: Vec<EmailStatus>,
    pub token: String,
}

#[derive(Serialize)]
pub struct EmailStatus {
    pub email: String,
    pub status: String,
}

/// Upload a public key to keys.openpgp.org (VKS API).
#[tauri::command]
pub async fn upload_to_keyserver(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<KeyserverUploadResult, String> {
    let armored = {
        let store = state.keystore.lock().map_err(|e| e.to_string())?;
        key::export_public_armored(&*store, &fingerprint).map_err(|e| e.to_string())?
    };
    let result = network::vks_upload_armored(&armored)
        .await
        .map_err(|e| e.to_string())?;
    Ok(KeyserverUploadResult {
        fingerprint: result.fingerprint,
        email_status: result
            .email_status
            .into_iter()
            .map(|e| EmailStatus {
                email: e.email,
                status: e.status,
            })
            .collect(),
        token: result.token,
    })
}

/// Request email verification for a specific email address on keys.openpgp.org.
#[tauri::command]
pub async fn request_keyserver_verification(
    token: String,
    email: String,
) -> Result<String, String> {
    network::request_verification(&token, &email)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Verification email sent to {}", email))
}
