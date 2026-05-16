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
    key, CipherSuite, KeyStore, KeySummary, KeyType, Passphrase, SubkeyFlags, SubkeySummary,
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

/// Wire-format summary row consumed by `KeyItem.vue` +
/// `KeyListMobile.vue`. Intentionally a subset of [`KeyInfo`] — the
/// per-subkey details + UID certifications aren't shown on the list
/// screen and would force an rpgp parse per key to populate.
#[derive(Serialize, Clone)]
pub struct KeyListRow {
    pub fingerprint: String,
    pub key_type: String,
    pub is_secret: bool,
    pub is_revoked: bool,
    pub creation_time: String,
    pub expiration_time: String,
    pub revocation_time: Option<String>,
    pub user_ids: Vec<UserIdData>,
    pub card_idents: Vec<String>,
}

fn summary_to_row(summary: &KeySummary, card_idents: Vec<String>) -> KeyListRow {
    // Parse "Name <email@example.com>" back into name / email pairs
    // so the frontend can render UID pills. Matches
    // `cert_info_to_key_info_inner`'s splitter, because the frontend
    // is already coded against that shape. `revoked` stays false
    // here — the `user_ids` SQL cache doesn't track per-UID
    // revocation, and the list view only surfaces whole-key
    // revocation anyway.
    let user_ids: Vec<UserIdData> = summary
        .user_ids
        .iter()
        .map(|uid| {
            let (name, email) = match uid.uid.find('<') {
                Some(lt_pos) => (
                    uid.uid[..lt_pos].trim().to_string(),
                    uid.uid[lt_pos + 1..]
                        .trim_end_matches('>')
                        .trim()
                        .to_string(),
                ),
                None => (uid.uid.clone(), uid.email.clone().unwrap_or_default()),
            };
            UserIdData {
                name,
                email,
                revoked: false,
                revocation_time: None,
            }
        })
        .collect();

    KeyListRow {
        fingerprint: summary.fingerprint.clone(),
        key_type: derive_key_type_label(&summary.subkeys),
        is_secret: summary.is_secret,
        is_revoked: summary.is_revoked,
        creation_time: summary
            .creation_time
            .map(|t| t.format("%d %b %Y").to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        expiration_time: summary
            .expiration_time
            .map(|t| t.format("%d %b %Y").to_string())
            .unwrap_or_else(|| "Never".to_string()),
        revocation_time: summary
            .revocation_time
            .map(|t| t.format("%d %b %Y %H:%M").to_string()),
        user_ids,
        card_idents,
    }
}

/// Pick the cipher-suite label the UI shows — mirrors
/// `cert_info_to_key_info_inner`'s logic so switching the list view
/// over to the summary payload doesn't cause visible tag changes.
fn derive_key_type_label(subkeys: &[SubkeySummary]) -> String {
    subkeys
        .iter()
        .find(|sk| sk.key_type != "unknown" && sk.key_type != "certification")
        .or_else(|| subkeys.first())
        .map(|sk| match (sk.algorithm.as_deref().unwrap_or(""), sk.bit_length.unwrap_or(0)) {
            ("RSA", n) if n >= 4096 => "RSA4096".to_string(),
            ("RSA", n) if n >= 2048 => "RSA2048".to_string(),
            ("RSA", n) => format!("RSA{}", n),
            ("EdDSA", _) | ("Ed25519", _) | ("ECDH", _) => "Cv25519".to_string(),
            ("ECDSA", 256) | ("ECDH P-256", _) => "NistP256".to_string(),
            ("ECDSA", 384) | ("ECDH P-384", _) => "NistP384".to_string(),
            ("ECDSA", 521) | ("ECDH P-521", _) => "NistP521".to_string(),
            (other, _) if !other.is_empty() => other.to_string(),
            _ => "Unknown".to_string(),
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Cheap alternative to `list_keys` for the list / gallery views.
///
/// Reads only the SQL summary columns (schema v4+) — no rpgp parse
/// per key. Card idents are fetched in a single batched query via
/// `libtumpa::card::link::card_idents_map`, killing the per-key N+1
/// the old command had on desktop.
#[tauri::command]
pub fn list_keys_summary(state: State<'_, AppState>) -> Result<Vec<KeyListRow>, String> {
    #[cfg(debug_assertions)]
    let t0 = std::time::Instant::now();
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    #[cfg(debug_assertions)]
    let t_lock = t0.elapsed();
    let mut summaries = store.list_keys_summary().map_err(|e| e.to_string())?;
    #[cfg(debug_assertions)]
    let t_summary = t0.elapsed();
    summaries.sort_by(|a, b| b.creation_time.cmp(&a.creation_time));

    // Desktop: one SQL query → fingerprint → card idents map.
    // Mobile: PCSC isn't available and link::card_idents_map isn't
    // compiled in, so every key gets an empty ident list.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let card_map = libtumpa::card::link::card_idents_map(&*store).unwrap_or_default();
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let card_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    #[cfg(debug_assertions)]
    let t_card = t0.elapsed();

    let rows: Vec<KeyListRow> = summaries
        .into_iter()
        .map(|s| {
            let idents = card_map.get(&s.fingerprint).cloned().unwrap_or_default();
            summary_to_row(&s, idents)
        })
        .collect();
    #[cfg(debug_assertions)]
    {
        let t_rows = t0.elapsed();
        eprintln!(
            "[tumpa/perf] list_keys_summary: n={} lock={:?} summary_sql={:?} card_map={:?} format={:?} total={:?}",
            rows.len(),
            t_lock,
            t_summary - t_lock,
            t_card - t_summary,
            t_rows - t_card,
            t_rows,
        );
    }
    Ok(rows)
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

    // libtumpa::key::generate always produces a V4 key; Cv25519Modern under
    // V4 yields Ed25519 (RFC 9580) + X25519, which Nitrokey 3 accepts for
    // upload and use. The legacy Cv25519 stays the default.
    let cipher = match cipher_suite.to_lowercase().as_str() {
        "rsa4096" | "rsa4k" => CipherSuite::Rsa4k,
        "cv25519modern" | "curve25519modern" | "x25519" => CipherSuite::Cv25519Modern,
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

// Takes raw file bytes rather than a path because Android's file picker
// returns SAF `content://` URIs that `std::fs::read` can't resolve. The
// frontend reads the file via `@tauri-apps/plugin-fs` (which handles both
// content URIs and desktop paths) and passes the bytes here.
#[tauri::command]
pub fn import_public_key(
    state: State<'_, AppState>,
    data: Vec<u8>,
) -> Result<KeyInfo, String> {
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

/// Return the armored public key for a fingerprint without touching
/// the filesystem. Used by the mobile UI, which can't call
/// `export_public_key` directly — on Android the dialog plugin returns
/// a `content://` URI instead of a filesystem path, and
/// `std::fs::write` quietly fails on it (Android creates a 0-byte
/// placeholder first, so the user sees an empty file + an error).
/// The JS side calls this, then uses `@tauri-apps/plugin-fs`'s
/// `writeTextFile` which does understand content URIs.
#[tauri::command]
pub fn get_public_armored(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<String, String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    key::export_public_armored(&*store, &fingerprint).map_err(|e| e.to_string())
}

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
