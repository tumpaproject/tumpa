use serde::Serialize;
use tauri::State;
use wecanencrypt::{
    create_key, parse_cert_bytes,
    CipherSuite, SubkeyFlags, KeyType,
};
use chrono::{Utc, NaiveDate, TimeZone};

use super::AppState;

#[derive(Serialize, Clone)]
pub struct KeyInfo {
    pub fingerprint: String,
    pub key_id: String,
    pub creation_time: String,
    pub expiration_time: String,
    pub user_ids: Vec<UserIdData>,
    pub is_secret: bool,
    pub subkeys: Vec<SubkeyData>,
}

#[derive(Serialize, Clone)]
pub struct UserIdData {
    pub name: String,
    pub email: String,
    pub revoked: bool,
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
    pub encryption: bool,
    pub signing: bool,
    pub authentication: bool,
}

fn cert_info_to_key_info(info: &wecanencrypt::CertificateInfo) -> KeyInfo {
    let user_ids: Vec<UserIdData> = info.user_ids.iter().map(|uid| {
        // Parse "Name <email>" format
        let uid_str = &uid.value;
        let (name, email) = if let Some(lt_pos) = uid_str.find('<') {
            let name = uid_str[..lt_pos].trim().to_string();
            let email = uid_str[lt_pos+1..].trim_end_matches('>').trim().to_string();
            (name, email)
        } else {
            (uid_str.clone(), String::new())
        };
        UserIdData {
            name,
            email,
            revoked: uid.revoked,
        }
    }).collect();

    let subkeys: Vec<SubkeyData> = info.subkeys.iter().map(|sk| {
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
            expiration_time: sk.expiration_time
                .map(|t| t.format("%d %b %Y").to_string())
                .unwrap_or_else(|| "Never".to_string()),
            is_revoked: sk.is_revoked,
        }
    }).collect();

    KeyInfo {
        fingerprint: info.fingerprint.clone(),
        key_id: info.key_id.clone(),
        creation_time: info.creation_time.format("%d %b %Y").to_string(),
        expiration_time: info.expiration_time
            .map(|t| t.format("%d %b %Y").to_string())
            .unwrap_or_else(|| "Never".to_string()),
        user_ids,
        is_secret: info.is_secret,
        subkeys,
    }
}

#[tauri::command]
pub fn list_keys(state: State<'_, AppState>) -> Result<Vec<KeyInfo>, String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let certs = store.list_certs().map_err(|e| e.to_string())?;
    Ok(certs.iter().map(cert_info_to_key_info).collect())
}

#[tauri::command]
pub fn generate_key(
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
    // Build UIDs
    let uids: Vec<String> = emails.iter()
        .map(|email| format!("{} <{}>", name, email))
        .collect();
    let uid_refs: Vec<&str> = uids.iter().map(|s| s.as_str()).collect();

    // Parse cipher suite
    let cipher = match cipher_suite.to_lowercase().as_str() {
        "rsa4096" | "rsa4k" => CipherSuite::Rsa4k,
        _ => CipherSuite::Cv25519,
    };

    // Parse expiry as DateTime<Utc>
    let expiry = if let Some(date_str) = expiry_date {
        if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Some(Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap()))
        } else {
            None
        }
    } else {
        None
    };

    let subkey_flags = SubkeyFlags {
        encryption,
        signing,
        authentication,
    };

    // Generate the key
    let generated = create_key(
        &password,
        &uid_refs,
        cipher,
        None, // creation time
        expiry, // primary expiry
        expiry, // subkey expiry
        subkey_flags,
        true, // can primary sign
        true, // include authentication subkey based on flag
    ).map_err(|e| format!("Key generation failed: {}", e))?;

    // Import into keystore
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let fp = store.import_cert(&generated.secret_key)
        .map_err(|e| format!("Failed to store key: {}", e))?;

    // Get full info
    let info = store.get_cert_info(&fp)
        .map_err(|e| format!("Failed to read key info: {}", e))?;

    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn import_key(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<KeyInfo, String> {
    // Read and parse to check if it's a secret key
    let data = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let cert_info = parse_cert_bytes(&data, true)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

    if !cert_info.is_secret {
        return Err("Please select a private key file.".to_string());
    }

    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let fp = store.import_cert(&data)
        .map_err(|e| format!("Failed to import key: {}", e))?;

    let info = store.get_cert_info(&fp)
        .map_err(|e| format!("Failed to read key info: {}", e))?;

    Ok(cert_info_to_key_info(&info))
}

#[tauri::command]
pub fn delete_key(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<(), String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    store.delete_cert(&fingerprint)
        .map_err(|e| format!("Failed to delete key: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn export_public_key(
    state: State<'_, AppState>,
    fingerprint: String,
    file_path: String,
) -> Result<(), String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let armored = store.export_cert_armored(&fingerprint)
        .map_err(|e| format!("Failed to export key: {}", e))?;
    drop(store);

    std::fs::write(&file_path, armored)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_available_subkeys(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<SubkeyAvailability, String> {
    let store = state.keystore.lock().map_err(|e| e.to_string())?;
    let info = store.get_cert_info(&fingerprint)
        .map_err(|e| format!("Key not found: {}", e))?;

    let now = Utc::now();
    let mut availability = SubkeyAvailability {
        encryption: false,
        signing: false,
        authentication: false,
    };

    for sk in &info.subkeys {
        // Skip revoked or expired subkeys
        if sk.is_revoked {
            continue;
        }
        if let Some(exp) = sk.expiration_time {
            if exp < now {
                continue;
            }
        }
        match sk.key_type {
            KeyType::Encryption => availability.encryption = true,
            KeyType::Signing => availability.signing = true,
            KeyType::Authentication => availability.authentication = true,
            _ => {}
        }
    }

    Ok(availability)
}
