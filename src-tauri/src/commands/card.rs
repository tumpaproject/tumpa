use serde::Serialize;
use tauri::State;
use wecanencrypt::{
    parse_cert_bytes, update_password, KeyType,
    card::{
        is_card_connected as card_connected,
        list_all_cards as card_list_all,
        get_card_details as card_details,
        reset_card,
        upload_primary_key_to_card,
        upload_key_to_card as card_upload_key,
        upload_subkey_by_fingerprint,
        change_user_pin as card_change_user_pin,
        change_admin_pin as card_change_admin_pin,
        set_cardholder_name as card_set_name,
        set_public_key_url as card_set_url,
        CardKeySlot,
    },
};

use super::AppState;

/// Default Yubikey PINs (after factory reset)
const DEFAULT_ADMIN_PIN: &[u8] = b"12345678";

#[derive(Serialize)]
pub struct CardSummaryInfo {
    pub ident: String,
    pub manufacturer_name: String,
    pub serial_number: String,
    pub cardholder_name: Option<String>,
}

#[derive(Serialize)]
pub struct CardDetails {
    pub ident: String,
    pub serial_number: String,
    pub cardholder_name: String,
    pub public_key_url: String,
    pub pin_retry_counter: u8,
    pub reset_code_retry_counter: u8,
    pub admin_pin_retry_counter: u8,
    pub signature_fingerprint: Option<String>,
    pub encryption_fingerprint: Option<String>,
    pub authentication_fingerprint: Option<String>,
    pub manufacturer: Option<String>,
    pub manufacturer_name: Option<String>,
}

#[tauri::command]
pub fn is_card_connected() -> bool {
    card_connected()
}

#[tauri::command]
pub fn list_cards() -> Result<Vec<CardSummaryInfo>, String> {
    let cards = card_list_all()
        .map_err(|e| format!("Failed to list cards: {}", e))?;

    Ok(cards.into_iter().map(|c| CardSummaryInfo {
        ident: c.ident,
        manufacturer_name: c.manufacturer_name,
        serial_number: c.serial_number,
        cardholder_name: c.cardholder_name,
    }).collect())
}

#[tauri::command]
pub fn get_card_details(ident: Option<String>) -> Result<CardDetails, String> {
    let info = card_details(ident.as_deref())
        .map_err(|e| format!("Failed to read card: {}", e))?;

    Ok(CardDetails {
        ident: info.ident,
        serial_number: info.serial_number,
        cardholder_name: info.cardholder_name.unwrap_or_default(),
        public_key_url: info.public_key_url.unwrap_or_default(),
        pin_retry_counter: info.pin_retry_counter,
        reset_code_retry_counter: info.reset_code_retry_counter,
        admin_pin_retry_counter: info.admin_pin_retry_counter,
        signature_fingerprint: info.signature_fingerprint,
        encryption_fingerprint: info.encryption_fingerprint,
        authentication_fingerprint: info.authentication_fingerprint,
        manufacturer: info.manufacturer,
        manufacturer_name: info.manufacturer_name,
    })
}

/// Upload key to smartcard.
///
/// `which_subkeys` is a bitmask:
///   1 = encryption subkey
///   2 = primary key to signing slot
///   4 = authentication subkey
///   8 = signing subkey to signing slot (mutually exclusive with 2)
///
/// The card is reset to factory defaults before upload.
#[tauri::command]
pub async fn upload_key_to_card(
    state: State<'_, AppState>,
    fingerprint: String,
    password: String,
    which_subkeys: u8,
) -> Result<(), String> {
    if !card_connected() {
        return Err("No smartcard connected.".to_string());
    }

    if which_subkeys & 2 != 0 && which_subkeys & 8 != 0 {
        return Err("Cannot upload both primary key and signing subkey to the signing slot.".to_string());
    }

    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let (cert_data, _) = store.get_cert(&fingerprint)
        .map_err(|e| format!("Key not found: {}", e))?;
    drop(store);

    let cert_info = parse_cert_bytes(&cert_data, true)
        .map_err(|e| format!("Failed to parse certificate: {}", e))?;

    // Verify password before touching the card — update_password with
    // same old/new password validates decryption without changing anything
    update_password(&cert_data, &password, &password)
        .map_err(|_| "Incorrect key password.".to_string())?;

    reset_card(None)
        .map_err(|e| format!("Failed to reset card: {}", e))?;

    if which_subkeys & 2 != 0 {
        upload_primary_key_to_card(
            &cert_data,
            password.as_bytes(),
            CardKeySlot::Signing,
            DEFAULT_ADMIN_PIN,
        ).map_err(|e| format!("Failed to upload primary key: {}", e))?;
    }

    if which_subkeys & 8 != 0 {
        let sign_sk = cert_info.subkeys.iter()
            .find(|sk| matches!(sk.key_type, KeyType::Signing))
            .ok_or("No signing subkey found")?;

        upload_subkey_by_fingerprint(
            &cert_data,
            password.as_bytes(),
            &sign_sk.fingerprint,
            CardKeySlot::Signing,
            DEFAULT_ADMIN_PIN,
        ).map_err(|e| format!("Failed to upload signing subkey: {}", e))?;
    }

    if which_subkeys & 1 != 0 {
        let _enc = cert_info.subkeys.iter()
            .find(|sk| matches!(sk.key_type, KeyType::Encryption))
            .ok_or("No encryption subkey found")?;

        card_upload_key(
            &cert_data,
            password.as_bytes(),
            CardKeySlot::Decryption,
            DEFAULT_ADMIN_PIN,
        ).map_err(|e| format!("Failed to upload encryption subkey: {}", e))?;
    }

    if which_subkeys & 4 != 0 {
        let auth = cert_info.subkeys.iter()
            .find(|sk| matches!(sk.key_type, KeyType::Authentication))
            .ok_or("No authentication subkey found")?;

        upload_subkey_by_fingerprint(
            &cert_data,
            password.as_bytes(),
            &auth.fingerprint,
            CardKeySlot::Authentication,
            DEFAULT_ADMIN_PIN,
        ).map_err(|e| format!("Failed to upload authentication subkey: {}", e))?;
    }

    // Auto-link card to key after successful upload
    if let Ok(details) = card_details(None) {
        let mut links = state.card_links.lock().map_err(|e| e.to_string())?;
        let entry = links.entry(fingerprint).or_insert_with(Vec::new);
        if !entry.contains(&details.ident) {
            entry.push(details.ident);
        }
        drop(links);
        let _ = state.save_card_links();
    }

    Ok(())
}

#[tauri::command]
pub fn update_card_name(name: String, admin_pin: String) -> Result<(), String> {
    if !card_connected() {
        return Err("No smartcard connected.".to_string());
    }
    card_set_name(&name, admin_pin.as_bytes(), None)
        .map_err(|e| format!("Failed to set name: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn update_card_url(url: String, admin_pin: String) -> Result<(), String> {
    if !card_connected() {
        return Err("No smartcard connected.".to_string());
    }
    card_set_url(&url, admin_pin.as_bytes(), None)
        .map_err(|e| format!("Failed to set URL: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn change_user_pin(admin_pin: String, new_pin: String) -> Result<(), String> {
    if !card_connected() {
        return Err("No smartcard connected.".to_string());
    }
    if new_pin.len() < 6 {
        return Err("User PIN must be at least 6 characters.".to_string());
    }
    card_change_user_pin(admin_pin.as_bytes(), new_pin.as_bytes(), None)
        .map_err(|e| format!("Failed to change user PIN: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn change_admin_pin(current_pin: String, new_pin: String) -> Result<(), String> {
    if !card_connected() {
        return Err("No smartcard connected.".to_string());
    }
    if new_pin.len() < 8 {
        return Err("Admin PIN must be at least 8 characters.".to_string());
    }
    card_change_admin_pin(current_pin.as_bytes(), new_pin.as_bytes(), None)
        .map_err(|e| format!("Failed to change admin PIN: {}", e))?;
    Ok(())
}

/// Link a card ident to a key fingerprint. Supports multiple cards per key.
#[tauri::command]
pub fn link_card_to_key(
    state: State<'_, AppState>,
    fingerprint: String,
    card_ident: String,
) -> Result<(), String> {
    let mut links = state.card_links.lock().map_err(|e| e.to_string())?;
    let entry = links.entry(fingerprint).or_insert_with(Vec::new);
    if !entry.contains(&card_ident) {
        entry.push(card_ident);
    }
    drop(links);
    state.save_card_links()
}

/// Remove a card ident from a key's associations.
#[tauri::command]
pub fn unlink_card_from_key(
    state: State<'_, AppState>,
    fingerprint: String,
    card_ident: String,
) -> Result<(), String> {
    let mut links = state.card_links.lock().map_err(|e| e.to_string())?;
    if let Some(entry) = links.get_mut(&fingerprint) {
        entry.retain(|c| c != &card_ident);
        if entry.is_empty() {
            links.remove(&fingerprint);
        }
    }
    drop(links);
    state.save_card_links()
}

/// Scan connected cards and match their key fingerprints against the keystore.
/// Returns detected associations that can be linked.
#[derive(Serialize)]
pub struct CardKeyMatch {
    pub key_fingerprint: String,
    pub card_ident: String,
    pub card_name: String,
}

#[tauri::command]
pub fn auto_detect_card_links(
    state: State<'_, AppState>,
) -> Result<Vec<CardKeyMatch>, String> {
    let cards = card_list_all()
        .map_err(|e| format!("Failed to list cards: {}", e))?;

    let mut matches = Vec::new();

    for card_summary in &cards {
        // Get detailed card info to read key slot fingerprints
        let info = card_details(Some(&card_summary.ident))
            .map_err(|e| format!("Failed to read card {}: {}", card_summary.ident, e))?;

        let card_fps: Vec<&str> = [
            info.signature_fingerprint.as_deref(),
            info.encryption_fingerprint.as_deref(),
            info.authentication_fingerprint.as_deref(),
        ].into_iter().flatten().collect();

        if card_fps.is_empty() {
            continue;
        }

        // Check against all keys in the keystore
        let store = state.keystore.lock().map_err(|e| e.to_string())?;
        let certs = store.list_certs().map_err(|e| e.to_string())?;
        drop(store);

        for cert in &certs {
            // Check if any subkey fingerprint matches a card slot
            let key_matches = cert.subkeys.iter().any(|sk| {
                card_fps.iter().any(|cfp| {
                    cfp.to_lowercase() == sk.fingerprint.to_lowercase()
                })
            });
            // Also check primary key fingerprint
            let primary_matches = card_fps.iter().any(|cfp| {
                cfp.to_lowercase() == cert.fingerprint.to_lowercase()
            });

            if key_matches || primary_matches {
                matches.push(CardKeyMatch {
                    key_fingerprint: cert.fingerprint.clone(),
                    card_ident: card_summary.ident.clone(),
                    card_name: card_summary.manufacturer_name.clone(),
                });
            }
        }
    }

    Ok(matches)
}

/// Update primary + all subkey expiry using the smartcard (PIN, not passphrase).
#[tauri::command]
pub fn update_key_expiry_on_card(
    state: State<'_, AppState>,
    fingerprint: String,
    new_date: String,
    pin: String,
) -> Result<super::keystore::KeyInfo, String> {
    use chrono::{NaiveDate, Utc};
    use wecanencrypt::card::{
        update_primary_expiry_on_card,
        update_subkeys_expiry_on_card,
    };

    let expiry_date = NaiveDate::parse_from_str(&new_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date: {}", e))?;
    let now = Utc::now();
    let expiry_dt = expiry_date.and_hms_opt(0, 0, 0).unwrap();
    let expiry_utc = chrono::TimeZone::from_utc_datetime(&Utc, &expiry_dt);
    let seconds = (expiry_utc - now).num_seconds();
    if seconds <= 0 {
        return Err("Expiry date must be in the future.".to_string());
    }
    let expiry_secs = seconds as u64;

    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let (_cert_data, info) = store.get_cert(&fingerprint)
        .map_err(|e| format!("Key not found: {}", e))?;

    // Get public key for card operations
    let armored = store.export_cert_armored(&fingerprint)
        .map_err(|e| format!("Failed to export key: {}", e))?;
    drop(store);

    // Update primary expiry via card
    let updated = update_primary_expiry_on_card(armored.as_bytes(), expiry_secs, pin.as_bytes())
        .map_err(|e| format!("Failed to update primary expiry on card: {}", e))?;

    // Update subkey expiries
    let subkey_fps: Vec<String> = info.subkeys.iter()
        .filter(|sk| sk.key_type != KeyType::Certification)
        .map(|sk| sk.fingerprint.clone())
        .collect();

    let final_cert = if !subkey_fps.is_empty() {
        let fp_refs: Vec<&str> = subkey_fps.iter().map(|s| s.as_str()).collect();
        update_subkeys_expiry_on_card(&updated, &fp_refs, expiry_secs, pin.as_bytes())
            .map_err(|e| format!("Failed to update subkey expiry on card: {}", e))?
    } else {
        updated
    };

    // Update in keystore
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    store.update_cert(&fingerprint, &final_cert)
        .map_err(|e| format!("Failed to update key: {}", e))?;

    let new_info = store.get_cert_info(&fingerprint)
        .map_err(|e| format!("Failed to read key info: {}", e))?;
    Ok(super::keystore::cert_info_to_key_info_with_cards(&new_info, &state))
}

/// Update selected subkey expiry using the smartcard (PIN, not passphrase).
#[tauri::command]
pub fn update_selected_subkeys_expiry_on_card(
    state: State<'_, AppState>,
    fingerprint: String,
    subkey_fingerprints: Vec<String>,
    new_date: String,
    pin: String,
) -> Result<super::keystore::KeyInfo, String> {
    use chrono::{NaiveDate, Utc};
    use wecanencrypt::card::update_subkeys_expiry_on_card;

    let expiry_date = NaiveDate::parse_from_str(&new_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date: {}", e))?;
    let now = Utc::now();
    let expiry_dt = expiry_date.and_hms_opt(0, 0, 0).unwrap();
    let expiry_utc = chrono::TimeZone::from_utc_datetime(&Utc, &expiry_dt);
    let seconds = (expiry_utc - now).num_seconds();
    if seconds <= 0 {
        return Err("Expiry date must be in the future.".to_string());
    }

    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let armored = store.export_cert_armored(&fingerprint)
        .map_err(|e| format!("Failed to export key: {}", e))?;
    drop(store);

    let fp_refs: Vec<&str> = subkey_fingerprints.iter().map(|s| s.as_str()).collect();
    let updated = update_subkeys_expiry_on_card(armored.as_bytes(), &fp_refs, seconds as u64, pin.as_bytes())
        .map_err(|e| format!("Failed to update subkey expiry on card: {}", e))?;

    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    store.update_cert(&fingerprint, &updated)
        .map_err(|e| format!("Failed to update key: {}", e))?;

    let new_info = store.get_cert_info(&fingerprint)
        .map_err(|e| format!("Failed to read key info: {}", e))?;
    Ok(super::keystore::cert_info_to_key_info_with_cards(&new_info, &state))
}
