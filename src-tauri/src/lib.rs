mod commands;

use commands::AppState;
use tauri::Manager;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use commands::{
    list_keys, generate_key, import_key, import_public_key, delete_key,
    export_public_key, get_available_subkeys,
    get_key_details, add_user_id, revoke_user_id, update_key_expiry,
    update_selected_subkeys_expiry, change_key_password, revoke_key_cmd,
    upload_to_keyserver, request_keyserver_verification,
    is_card_connected, list_cards, get_card_details,
    upload_key_to_card, update_card_name, update_card_url,
    change_user_pin, change_admin_pin,
    link_card_to_key, unlink_card_from_key, auto_detect_card_links,
    update_key_expiry_on_card, update_selected_subkeys_expiry_on_card,
    get_card_touch_modes, set_card_touch_mode,
};

#[cfg(any(target_os = "android", target_os = "ios"))]
use commands::{
    list_keys, generate_key, import_key, import_public_key, delete_key,
    export_public_key,
    get_key_details, add_user_id, revoke_user_id, update_key_expiry,
    update_selected_subkeys_expiry, change_key_password, revoke_key_cmd,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = resolve_data_dir(app)?;
            if !data_dir.exists() {
                std::fs::create_dir_all(&data_dir)?;
            }
            let db_path = data_dir.join("keys.db");
            let db_path_str = db_path.to_str().ok_or("invalid db path")?;
            app.manage(AppState::new(db_path_str, &data_dir));
            Ok(())
        });

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        list_keys,
        generate_key,
        import_key,
        import_public_key,
        delete_key,
        export_public_key,
        get_available_subkeys,
        get_key_details,
        add_user_id,
        revoke_user_id,
        update_key_expiry,
        update_selected_subkeys_expiry,
        change_key_password,
        revoke_key_cmd,
        upload_to_keyserver,
        request_keyserver_verification,
        is_card_connected,
        list_cards,
        get_card_details,
        upload_key_to_card,
        update_card_name,
        update_card_url,
        change_user_pin,
        change_admin_pin,
        link_card_to_key,
        unlink_card_from_key,
        auto_detect_card_links,
        update_key_expiry_on_card,
        update_selected_subkeys_expiry_on_card,
        get_card_touch_modes,
        set_card_touch_mode,
    ]);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        list_keys,
        generate_key,
        import_key,
        import_public_key,
        delete_key,
        export_public_key,
        get_key_details,
        add_user_id,
        revoke_user_id,
        update_key_expiry,
        update_selected_subkeys_expiry,
        change_key_password,
        revoke_key_cmd,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn resolve_data_dir(_app: &tauri::App) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // Desktop: keep the historical ~/.tumpa/ location so existing installs keep working.
    let home = dirs::home_dir().ok_or("could not determine home directory")?;
    Ok(home.join(".tumpa"))
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn resolve_data_dir(app: &tauri::App) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // Mobile: use the OS-assigned per-app data directory (sandboxed).
    Ok(app.path().app_data_dir()?)
}
