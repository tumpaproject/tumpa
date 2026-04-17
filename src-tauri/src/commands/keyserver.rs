use serde::{Deserialize, Serialize};
use tauri::State;

use super::AppState;

#[derive(Deserialize)]
struct VksUploadResponse {
    key_fpr: String,
    status: std::collections::HashMap<String, String>,
    token: String,
}

#[derive(Serialize)]
pub struct KeyserverUploadResult {
    pub fingerprint: String,
    /// Per-email verification status: "unpublished", "published", "revoked", "pending"
    pub email_status: Vec<EmailStatus>,
    pub token: String,
}

#[derive(Serialize)]
pub struct EmailStatus {
    pub email: String,
    pub status: String,
}

/// Upload a public key to keys.openpgp.org (VKS API).
///
/// After uploading, each email address needs to be verified separately
/// via a confirmation email sent by the server.
#[tauri::command]
pub async fn upload_to_keyserver(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<KeyserverUploadResult, String> {
    let armored = {
        let store = state.keystore.lock().map_err(|e| e.to_string())?;
        store.export_key_armored(&fingerprint)
            .map_err(|e| format!("Failed to export key: {}", e))?
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let body = serde_json::json!({
        "keytext": armored,
    });

    let response = client
        .post("https://keys.openpgp.org/vks/v1/upload")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to upload key: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Keyserver returned {}: {}", status, body));
    }

    let vks: VksUploadResponse = response.json().await
        .map_err(|e| format!("Failed to parse keyserver response: {}", e))?;

    let email_status: Vec<EmailStatus> = vks.status.into_iter()
        .map(|(email, status)| EmailStatus { email, status })
        .collect();

    Ok(KeyserverUploadResult {
        fingerprint: vks.key_fpr,
        email_status,
        token: vks.token,
    })
}

/// Request email verification for a specific email address on keys.openpgp.org.
///
/// This sends a verification email to the specified address.
#[tauri::command]
pub async fn request_keyserver_verification(
    token: String,
    email: String,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let body = serde_json::json!({
        "token": token,
        "addresses": [email],
    });

    let response = client
        .post("https://keys.openpgp.org/vks/v1/request-verify")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to request verification: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        return Err(format!("Keyserver returned {}: {}", status, body_text));
    }

    Ok(format!("Verification email sent to {}", email))
}
