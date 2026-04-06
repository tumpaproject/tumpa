mod commands;

use commands::{
    list_keys, generate_key, import_key, delete_key,
    export_public_key, get_available_subkeys,
    get_key_details, add_user_id, revoke_user_id, update_key_expiry,
    is_card_connected, list_cards, get_card_details,
    upload_key_to_card, update_card_name, update_card_url,
    change_user_pin, change_admin_pin,
    AppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Determine keystore path
    let home = dirs::home_dir().expect("Could not determine home directory");
    let tumpa_dir = home.join(".tumpa");

    // Create ~/.tumpa/ if it doesn't exist
    if !tumpa_dir.exists() {
        std::fs::create_dir_all(&tumpa_dir).expect("Failed to create ~/.tumpa directory");
    }

    let db_path = tumpa_dir.join("keys.db");
    let db_path_str = db_path.to_str().expect("Invalid path");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new(db_path_str))
        .invoke_handler(tauri::generate_handler![
            list_keys,
            generate_key,
            import_key,
            delete_key,
            export_public_key,
            get_available_subkeys,
            get_key_details,
            add_user_id,
            revoke_user_id,
            update_key_expiry,
            is_card_connected,
            list_cards,
            get_card_details,
            upload_key_to_card,
            update_card_name,
            update_card_url,
            change_user_pin,
            change_admin_pin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
