//! Smartcard-backed `#[tauri::command]` functions.
//!
//! All card logic lives in `libtumpa::card`; this module is just a thin
//! IPC shell that serializes arguments, wraps PINs/passphrases in
//! zeroing containers, and reshapes libtumpa results into the JSON
//! structs the frontend expects.
//!
//! Works on both desktop and mobile. On desktop the card backend is
//! PC/SC; on mobile it's the tauri-plugin-tumpa-card bridge registered
//! via `card_bridge::register_backend_provider`. Enumeration APIs
//! (`is_card_connected`, `list_cards`, `auto_detect_card_links`) are
//! PCSC-only — mobile skips those and drives sessions explicitly from
//! the UI.

use libtumpa::card::{
    admin, link,
    upload::{self, flags as upload_flags},
    CardInfo, KeySlot, TouchMode,
};
use libtumpa::{Passphrase, Pin};
use serde::Serialize;
use tauri::State;

use super::AppState;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub fn is_card_connected() -> bool {
    libtumpa::card::is_card_connected()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub fn list_cards() -> Result<Vec<CardSummaryInfo>, String> {
    let cards = libtumpa::card::list_all_cards()
        .map_err(|e| format!("Failed to list cards: {}", e))?;
    Ok(cards
        .into_iter()
        .map(|c| CardSummaryInfo {
            ident: c.ident,
            manufacturer_name: c.manufacturer_name,
            serial_number: c.serial_number,
            cardholder_name: c.cardholder_name,
        })
        .collect())
}

fn card_info_to_details(info: CardInfo) -> CardDetails {
    CardDetails {
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
    }
}

#[tauri::command]
pub fn get_card_details(ident: Option<String>) -> Result<CardDetails, String> {
    let info = libtumpa::card::get_card_details(ident.as_deref())
        .map_err(|e| format!("Failed to read card: {}", e))?;
    Ok(card_info_to_details(info))
}

/// Upload key to smartcard.
///
/// `which_subkeys` is a bitmask (matches `libtumpa::card::upload::flags`):
///   1 = encryption subkey
///   2 = primary key to signing slot
///   4 = authentication subkey
///   8 = signing subkey to signing slot (mutually exclusive with 2)
///
/// libtumpa handles: passphrase verification, factory reset, per-slot
/// upload, and auto-linking the card to the key in the `card_keys` table.
#[tauri::command]
pub async fn upload_key_to_card(
    state: State<'_, AppState>,
    fingerprint: String,
    password: String,
    which_subkeys: u8,
) -> Result<(), String> {
    let pw = Passphrase::new(password);
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    upload::upload(&*store, &fingerprint, &pw, which_subkeys, None).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_card_name(name: String, admin_pin: String) -> Result<(), String> {
    let pin = Pin::new(admin_pin.into_bytes());
    admin::set_cardholder_name(&name, &pin, None).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_card_url(url: String, admin_pin: String) -> Result<(), String> {
    let pin = Pin::new(admin_pin.into_bytes());
    admin::set_public_key_url(&url, &pin, None).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn change_user_pin(admin_pin: String, new_pin: String) -> Result<(), String> {
    let admin = Pin::new(admin_pin.into_bytes());
    let new = Pin::new(new_pin.into_bytes());
    admin::change_user_pin(&admin, &new, None).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn change_admin_pin(current_pin: String, new_pin: String) -> Result<(), String> {
    let current = Pin::new(current_pin.into_bytes());
    let new = Pin::new(new_pin.into_bytes());
    admin::change_admin_pin(&current, &new, None).map_err(|e| e.to_string())
}

/// Link a card ident to a key fingerprint.
///
/// Thin wrapper: libtumpa requires a `CardInfo` so it can record the card
/// serial + manufacturer alongside the link. We read card details once to
/// populate that, then call `libtumpa::card::link::auto_link_after_upload`
/// which writes one row per slot fingerprint.
#[tauri::command]
pub fn link_card_to_key(
    state: State<'_, AppState>,
    fingerprint: String,
    card_ident: String,
) -> Result<(), String> {
    let info = libtumpa::card::get_card_details(Some(&card_ident))
        .map_err(|e| format!("Failed to read card: {}", e))?;
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    link::auto_link_after_upload(&*store, &info, &fingerprint).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unlink_card_from_key(
    state: State<'_, AppState>,
    fingerprint: String,
    card_ident: String,
) -> Result<(), String> {
    let _ = fingerprint; // libtumpa unlinks the card across all its rows
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    link::unlink_card(&*store, &card_ident).map_err(|e| e.to_string())
}

/// Wire shape for auto-detect results. Matches the desktop frontend's
/// existing expectations from pre-libtumpa days.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Serialize)]
pub struct CardKeyMatch {
    pub key_fingerprint: String,
    pub card_ident: String,
    pub card_name: String,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub fn auto_detect_card_links(
    state: State<'_, AppState>,
) -> Result<Vec<CardKeyMatch>, String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let detections = link::auto_detect(&*store).map_err(|e| e.to_string())?;
    Ok(detections
        .into_iter()
        .map(|d| CardKeyMatch {
            key_fingerprint: d.key_fingerprint,
            card_ident: d.card_ident,
            card_name: d.card_summary.manufacturer_name,
        })
        .collect())
}

/// Update the primary key + all non-certification subkeys' expiry using
/// a card PIN.
#[tauri::command]
pub fn update_key_expiry_on_card(
    state: State<'_, AppState>,
    fingerprint: String,
    new_date: String,
    pin: String,
) -> Result<super::keystore::KeyInfo, String> {
    use chrono::{NaiveDate, TimeZone, Utc};
    let expiry_date = NaiveDate::parse_from_str(&new_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date: {}", e))?;
    let expiry_dt = expiry_date.and_hms_opt(0, 0, 0).unwrap();
    let expiry_utc = Utc.from_utc_datetime(&expiry_dt);
    if expiry_utc <= Utc::now() {
        return Err("Expiry date must be in the future.".to_string());
    }
    let card_pin = Pin::new(pin.into_bytes());
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = libtumpa::card::expiry::update_key_expiry_on_card(
        &*store,
        &fingerprint,
        expiry_utc,
        &card_pin,
        None,
    )
    .map_err(|e| e.to_string())?;
    Ok(super::keystore::cert_info_to_key_info_with_cards(
        &info, &*store,
    ))
}

#[tauri::command]
pub fn update_selected_subkeys_expiry_on_card(
    state: State<'_, AppState>,
    fingerprint: String,
    subkey_fingerprints: Vec<String>,
    new_date: String,
    pin: String,
) -> Result<super::keystore::KeyInfo, String> {
    use chrono::{NaiveDate, TimeZone, Utc};
    let expiry_date = NaiveDate::parse_from_str(&new_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date: {}", e))?;
    let expiry_dt = expiry_date.and_hms_opt(0, 0, 0).unwrap();
    let expiry_utc = Utc.from_utc_datetime(&expiry_dt);
    if expiry_utc <= Utc::now() {
        return Err("Expiry date must be in the future.".to_string());
    }
    let card_pin = Pin::new(pin.into_bytes());
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let fp_refs: Vec<&str> = subkey_fingerprints.iter().map(|s| s.as_str()).collect();
    let info = libtumpa::card::expiry::update_selected_subkeys_expiry_on_card(
        &*store,
        &fingerprint,
        &fp_refs,
        expiry_utc,
        &card_pin,
        None,
    )
    .map_err(|e| e.to_string())?;
    Ok(super::keystore::cert_info_to_key_info_with_cards(
        &info, &*store,
    ))
}

/// Touch mode info for a single slot.
#[derive(Serialize)]
pub struct SlotTouchInfo {
    pub slot: String,
    pub mode: String,
    pub is_fixed: bool,
    pub supported: bool,
}

#[tauri::command]
pub fn get_card_touch_modes() -> Result<Vec<SlotTouchInfo>, String> {
    let modes = admin::get_touch_modes(None).map_err(|e| e.to_string())?;

    fn mode_to_string(m: &TouchMode) -> String {
        match m {
            TouchMode::Off => "Off".to_string(),
            TouchMode::On => "On".to_string(),
            TouchMode::Fixed => "Fixed".to_string(),
            TouchMode::Cached => "Cached".to_string(),
            TouchMode::CachedFixed => "CachedFixed".to_string(),
        }
    }

    fn is_fixed(m: &TouchMode) -> bool {
        matches!(m, TouchMode::Fixed | TouchMode::CachedFixed)
    }

    fn slot_to_string(s: KeySlot) -> String {
        match s {
            KeySlot::Signature => "Signature".to_string(),
            KeySlot::Encryption => "Encryption".to_string(),
            KeySlot::Authentication => "Authentication".to_string(),
        }
    }

    Ok(modes
        .into_iter()
        .map(|m| SlotTouchInfo {
            slot: slot_to_string(m.slot),
            mode: m
                .mode
                .as_ref()
                .map(mode_to_string)
                .unwrap_or_else(|| "N/A".to_string()),
            is_fixed: m.mode.as_ref().map(is_fixed).unwrap_or(false),
            supported: m.mode.is_some(),
        })
        .collect())
}

#[tauri::command]
pub fn set_card_touch_mode(
    slot: String,
    mode: String,
    admin_pin: String,
) -> Result<(), String> {
    let key_slot = match slot.as_str() {
        "Signature" => KeySlot::Signature,
        "Encryption" => KeySlot::Encryption,
        "Authentication" => KeySlot::Authentication,
        _ => return Err(format!("Unknown slot: {}", slot)),
    };
    let touch_mode = match mode.as_str() {
        "Off" => TouchMode::Off,
        "On" => TouchMode::On,
        "Fixed" => TouchMode::Fixed,
        "Cached" => TouchMode::Cached,
        "CachedFixed" => TouchMode::CachedFixed,
        _ => return Err(format!("Unknown touch mode: {}", mode)),
    };
    let pin = Pin::new(admin_pin.into_bytes());
    admin::set_touch_mode(key_slot, touch_mode, &pin, None).map_err(|e| e.to_string())
}

/// Bitmask flag constants, kept in sync with `libtumpa::card::upload::flags`.
/// Left here as a reference for frontend callers; we don't actually expose
/// them over IPC.
#[allow(dead_code)]
const _WHICH_SUBKEYS_DOC: (u8, u8, u8, u8) = (
    upload_flags::ENCRYPTION,
    upload_flags::PRIMARY_TO_SIGNING,
    upload_flags::AUTHENTICATION,
    upload_flags::SIGNING_SUBKEY,
);
