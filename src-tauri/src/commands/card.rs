use serde::Serialize;

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
    wecanencrypt::card::is_card_connected()
}

#[tauri::command]
pub fn get_card_details() -> Result<CardDetails, String> {
    let info = wecanencrypt::card::get_card_details()
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
