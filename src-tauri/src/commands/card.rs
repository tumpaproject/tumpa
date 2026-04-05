use serde::Serialize;
use tauri::State;
use wecanencrypt::{
    parse_cert_bytes, KeyType,
    card::{
        is_card_connected as card_connected,
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
pub struct CardDetails {
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
}

#[tauri::command]
pub fn is_card_connected() -> bool {
    card_connected()
}

#[tauri::command]
pub fn get_card_details() -> Result<CardDetails, String> {
    let info = card_details()
        .map_err(|e| format!("Failed to read card: {}", e))?;

    Ok(CardDetails {
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

    // Validate: cannot upload both primary and signing subkey to signing slot
    if which_subkeys & 2 != 0 && which_subkeys & 8 != 0 {
        return Err("Cannot upload both primary key and signing subkey to the signing slot.".to_string());
    }

    // Get key from keystore
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let (cert_data, _) = store.get_cert(&fingerprint)
        .map_err(|e| format!("Key not found: {}", e))?;
    drop(store); // Release lock before card I/O

    // Parse cert to find subkeys
    let cert_info = parse_cert_bytes(&cert_data, true)
        .map_err(|e| format!("Failed to parse certificate: {}", e))?;

    // Reset card to factory defaults
    reset_card()
        .map_err(|e| format!("Failed to reset card: {}", e))?;

    // Upload primary key to Signing slot
    if which_subkeys & 2 != 0 {
        upload_primary_key_to_card(
            &cert_data,
            password.as_bytes(),
            CardKeySlot::Signing,
            DEFAULT_ADMIN_PIN,
        ).map_err(|e| format!("Failed to upload primary key: {}", e))?;
    }

    // Upload signing subkey to Signing slot
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

    // Upload encryption subkey
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

    // Upload authentication subkey
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

    Ok(())
}

#[tauri::command]
pub fn update_card_name(name: String, admin_pin: String) -> Result<(), String> {
    if !card_connected() {
        return Err("No smartcard connected.".to_string());
    }
    card_set_name(&name, admin_pin.as_bytes())
        .map_err(|e| format!("Failed to set name: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn update_card_url(url: String, admin_pin: String) -> Result<(), String> {
    if !card_connected() {
        return Err("No smartcard connected.".to_string());
    }
    card_set_url(&url, admin_pin.as_bytes())
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
    card_change_user_pin(admin_pin.as_bytes(), new_pin.as_bytes())
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
    card_change_admin_pin(current_pin.as_bytes(), new_pin.as_bytes())
        .map_err(|e| format!("Failed to change admin PIN: {}", e))?;
    Ok(())
}
