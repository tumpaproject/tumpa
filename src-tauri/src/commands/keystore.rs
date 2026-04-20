//! Keystore-backed #[tauri::command] functions.
//!
//! These are thin IPC shells that call into `libtumpa::key` for all the
//! OpenPGP logic. libtumpa handles the keystore mutation + readback; this
//! module just handles Tauri wiring, serialization shapes, and the small
//! conveniences the UI expects (combined name+email UID, date parsing,
//! `card_idents` annotation on `KeyInfo`).
//!
//! Secrets (`password`) are wrapped in `libtumpa::Passphrase` (a
//! `Zeroizing<String>`) at the entry point so the underlying buffer is
//! zeroed as soon as the command returns.

use chrono::{NaiveDate, TimeZone, Utc};
use libtumpa::{
    key, CipherSuite, KeyStore, KeyType, Passphrase, SubkeyFlags,
};
use serde::Serialize;
use tauri::State;

use super::AppState;

#[derive(Serialize, Clone)]
pub struct KeyInfo {
    pub fingerprint: String,
    pub key_id: String,
    pub creation_time: String,
    pub expiration_time: String,
    pub key_type: String,
    pub user_ids: Vec<UserIdData>,
    pub is_secret: bool,
    pub is_revoked: bool,
    pub revocation_time: Option<String>,
    pub card_idents: Vec<String>,
    pub subkeys: Vec<SubkeyData>,
}

#[derive(Serialize, Clone)]
pub struct UserIdData {
    pub name: String,
    pub email: String,
    pub revoked: bool,
    pub revocation_time: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct SubkeyData {
    pub fingerprint: String,
    pub key_type: String,
    pub creation_time: String,
    pub expiration_time: String,
    pub is_revoked: bool,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Serialize)]
pub struct SubkeyAvailability {
    pub primary_can_sign: bool,
    pub signing_subkey: bool,
    pub encryption: bool,
    pub authentication: bool,
}

fn cert_info_to_key_info(info: &libtumpa::KeyInfo) -> KeyInfo {
    cert_info_to_key_info_inner(info, vec![])
}

/// Build the wire `KeyInfo` with `card_idents` populated from the
/// keystore's `card_keys` table (desktop) or empty (mobile).
pub fn cert_info_to_key_info_with_cards(
    info: &libtumpa::KeyInfo,
    #[cfg(not(any(target_os = "android", target_os = "ios")))] store: &KeyStore,
    #[cfg(any(target_os = "android", target_os = "ios"))] _store: &KeyStore,
) -> KeyInfo {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let card_idents = libtumpa::card::link::card_idents_for_key(store, &info.fingerprint)
        .unwrap_or_default();
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let card_idents: Vec<String> = Vec::new();
    cert_info_to_key_info_inner(info, card_idents)
}

fn cert_info_to_key_info_inner(info: &libtumpa::KeyInfo, card_idents: Vec<String>) -> KeyInfo {
    let user_ids: Vec<UserIdData> = info
        .user_ids
        .iter()
        .map(|uid| {
            let uid_str = &uid.value;
            let (name, email) = if let Some(lt_pos) = uid_str.find('<') {
                let name = uid_str[..lt_pos].trim().to_string();
                let email = uid_str[lt_pos + 1..].trim_end_matches('>').trim().to_string();
                (name, email)
            } else {
                (uid_str.clone(), String::new())
            };
            UserIdData {
                name,
                email,
                revoked: uid.revoked,
                revocation_time: uid
                    .revocation_time
                    .map(|t| t.format("%d %b %Y %H:%M").to_string()),
            }
        })
        .collect();

    let subkeys: Vec<SubkeyData> = info
        .subkeys
        .iter()
        .map(|sk| {
            let key_type = match sk.key_type {
                KeyType::Encryption => "encryption",
                KeyType::Signing => "signing",
                KeyType::Authentication => "authentication",
                KeyType::Certification => "certification",
                KeyType::Unknown => "unknown",
            };
            SubkeyData {
                fingerprint: sk.fingerprint.clone(),
                key_type: key_type.to_string(),
                creation_time: sk.creation_time.format("%d %b %Y").to_string(),
                expiration_time: sk
                    .expiration_time
                    .map(|t| t.format("%d %b %Y").to_string())
                    .unwrap_or_else(|| "Never".to_string()),
                is_revoked: sk.is_revoked,
            }
        })
        .collect();

    // Derive the displayed "cipher suite" from the first identifiable subkey.
    let key_type = info
        .subkeys
        .iter()
        .find(|sk| sk.key_type != KeyType::Unknown)
        .map(|sk| match (sk.algorithm.as_str(), sk.bit_length) {
            ("RSA", n) if n >= 4096 => "RSA4096".to_string(),
            ("RSA", n) if n >= 2048 => "RSA2048".to_string(),
            ("RSA", n) => format!("RSA{}", n),
            ("EdDSA", _) | ("Ed25519", _) | ("ECDH", _) => "Cv25519".to_string(),
            ("ECDSA", 256) | ("ECDH P-256", _) => "NistP256".to_string(),
            ("ECDSA", 384) | ("ECDH P-384", _) => "NistP384".to_string(),
            ("ECDSA", 521) | ("ECDH P-521", _) => "NistP521".to_string(),
            (other, _) => other.to_string(),
        })
        .unwrap_or_else(|| "Unknown".to_string());

    KeyInfo {
        fingerprint: info.fingerprint.clone(),
        key_id: info.key_id.clone(),
        creation_time: info.creation_time.format("%d %b %Y").to_string(),
        expiration_time: info
            .expiration_time
            .map(|t| t.format("%d %b %Y").to_string())
            .unwrap_or_else(|| "Never".to_string()),
        key_type,
        user_ids,
        is_secret: info.is_secret,
        is_revoked: info.is_revoked,
        revocation_time: info
            .revocation_time
            .map(|t| t.format("%d %b %Y %H:%M").to_string()),
        card_idents,
        subkeys,
    }
}

fn parse_expiry(date_str: &str) -> Result<chrono::DateTime<Utc>, String> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date: {}", e))?;
    Ok(Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap()))
}

#[tauri::command]
pub fn list_keys(state: State<'_, AppState>) -> Result<Vec<KeyInfo>, String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let mut certs = store.list_keys().map_err(|e| e.to_string())?;
    certs.sort_by(|a, b| b.creation_time.cmp(&a.creation_time));
    Ok(certs
        .iter()
        .map(|c| cert_info_to_key_info_with_cards(c, &*store))
        .collect())
}

#[tauri::command]
pub async fn generate_key(
    state: State<'_, AppState>,
    name: String,
    emails: Vec<String>,
    password: String,
    expiry_date: Option<String>,
    encryption: bool,
    signing: bool,
    authentication: bool,
    cipher_suite: String,
) -> Result<KeyInfo, String> {
    let uids: Vec<String> = emails
        .iter()
        .map(|email| format!("{} <{}>", name, email))
        .collect();

    let cipher = match cipher_suite.to_lowercase().as_str() {
        "rsa4096" | "rsa4k" => CipherSuite::Rsa4k,
        _ => CipherSuite::Cv25519,
    };

    let expiry = match expiry_date.as_deref() {
        Some(s) if !s.is_empty() => Some(parse_expiry(s)?),
        _ => None,
    };

    let params = key::GenerateKeyParams {
        uids,
        cipher_suite: cipher,
        expiry,
        subkey_flags: SubkeyFlags {
            encryption,
            signing,
            authentication,
        },
        can_primary_sign: true,
    };

    // libtumpa::key::generate is CPU-heavy (RSA 4096 can take seconds).
    // Run it on a blocking-thread so the async runtime isn't stalled,
    // then import into the keystore on the command's worker thread.
    let pw = Passphrase::new(password);
    let generated = tokio::task::spawn_blocking(move || key::generate(params, &pw))
        .await
        .map_err(|e| format!("Key generation task failed: {}", e))?
        .map_err(|e| e.to_string())?;

    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let fp = store
        .import_key(&generated.secret_key)
        .map_err(|e| format!("Failed to store key: {}", e))?;
    let info = store
        .get_key_info(&fp)
        .map_err(|e| format!("Failed to read key info: {}", e))?;

    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn import_key(state: State<'_, AppState>, file_path: String) -> Result<KeyInfo, String> {
    let data = std::fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = key::import_secret(&*store, &data).map_err(|e| match e {
        libtumpa::Error::InvalidInput(s) if s.contains("not a secret") => {
            "Please select a private key file.".to_string()
        }
        other => other.to_string(),
    })?;
    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn import_public_key(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<KeyInfo, String> {
    let data = std::fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = key::import_any(&*store, &data).map_err(|e| e.to_string())?;
    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn delete_key(state: State<'_, AppState>, fingerprint: String) -> Result<(), String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    key::delete(&*store, &fingerprint).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_public_key(
    state: State<'_, AppState>,
    fingerprint: String,
    file_path: String,
) -> Result<(), String> {
    let armored = {
        let store = state.keystore.lock().map_err(|e| e.to_string())?;
        key::export_public_armored(&*store, &fingerprint).map_err(|e| e.to_string())?
    };
    std::fs::write(&file_path, armored).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub fn get_available_subkeys(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<SubkeyAvailability, String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let avail = key::available_subkeys(&*store, &fingerprint).map_err(|e| e.to_string())?;
    Ok(SubkeyAvailability {
        primary_can_sign: avail.primary_can_sign,
        signing_subkey: avail.signing_subkey,
        encryption: avail.encryption,
        authentication: avail.authentication,
    })
}

#[tauri::command]
pub fn get_key_details(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<KeyInfo, String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = store
        .get_key_info(&fingerprint)
        .map_err(|e| format!("Key not found: {}", e))?;
    Ok(cert_info_to_key_info_with_cards(&info, &*store))
}

#[tauri::command]
pub fn add_user_id(
    state: State<'_, AppState>,
    fingerprint: String,
    name: String,
    email: String,
    password: String,
) -> Result<KeyInfo, String> {
    let uid_str = format!("{} <{}>", name, email);
    let pw = Passphrase::new(password);
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = key::add_uid(&*store, &fingerprint, &uid_str, &pw).map_err(|e| e.to_string())?;
    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn revoke_user_id(
    state: State<'_, AppState>,
    fingerprint: String,
    uid: String,
    password: String,
) -> Result<KeyInfo, String> {
    let pw = Passphrase::new(password);
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = key::revoke_uid(&*store, &fingerprint, &uid, &pw).map_err(|e| e.to_string())?;
    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn update_key_expiry(
    state: State<'_, AppState>,
    fingerprint: String,
    new_date: String,
    password: String,
) -> Result<KeyInfo, String> {
    let expiry = parse_expiry(&new_date)?;
    let pw = Passphrase::new(password);
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = key::update_expiry(&*store, &fingerprint, expiry, &pw).map_err(|e| e.to_string())?;
    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn update_selected_subkeys_expiry(
    state: State<'_, AppState>,
    fingerprint: String,
    subkey_fingerprints: Vec<String>,
    new_date: String,
    password: String,
) -> Result<KeyInfo, String> {
    let expiry = parse_expiry(&new_date)?;
    let pw = Passphrase::new(password);
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let fp_refs: Vec<&str> = subkey_fingerprints.iter().map(|s| s.as_str()).collect();
    let info = key::update_subkey_expiry(&*store, &fingerprint, &fp_refs, expiry, &pw)
        .map_err(|e| e.to_string())?;
    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn change_key_password(
    state: State<'_, AppState>,
    fingerprint: String,
    old_password: String,
    new_password: String,
) -> Result<(), String> {
    let old_pw = Passphrase::new(old_password);
    let new_pw = Passphrase::new(new_password);
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    key::change_password(&*store, &fingerprint, &old_pw, &new_pw).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn revoke_key_cmd(
    state: State<'_, AppState>,
    fingerprint: String,
    password: String,
) -> Result<KeyInfo, String> {
    let pw = Passphrase::new(password);
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = key::revoke(&*store, &fingerprint, &pw).map_err(|e| e.to_string())?;
    Ok(cert_info_to_key_info(&info))
}
