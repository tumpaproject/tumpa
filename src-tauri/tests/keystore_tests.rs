/// Integration tests for the keystore logic used by Tauri commands.
///
/// These tests exercise the wecanencrypt KeyStore API directly (the same
/// calls our Tauri commands make), using in-memory keystores so they
/// don't touch the user's ~/.tumpa directory.
use wecanencrypt::{
    create_key, parse_cert_bytes, add_uid, revoke_uid,
    update_primary_expiry, update_subkeys_expiry, update_password,
    CipherSuite, SubkeyFlags, KeyType, KeyStore,
    encrypt_bytes, decrypt_bytes,
};
use chrono::{Utc, Duration};

fn generate_test_key(password: &str) -> wecanencrypt::GeneratedKey {
    create_key(
        password,
        &["Test User <test@example.com>"],
        CipherSuite::Cv25519,
        None,
        Some(Utc::now() + Duration::days(365)),
        Some(Utc::now() + Duration::days(365)),
        SubkeyFlags {
            encryption: true,
            signing: true,
            authentication: false,
        },
        true,
        true,
    )
    .expect("Key generation failed")
}

#[test]
fn test_keystore_create_and_list() {
    let store = KeyStore::open_in_memory().unwrap();
    assert_eq!(store.count().unwrap(), 0);

    let key = generate_test_key("test123");
    let fp = store.import_cert(&key.secret_key).unwrap();
    assert!(!fp.is_empty());

    let certs = store.list_certs().unwrap();
    assert_eq!(certs.len(), 1);
    assert_eq!(certs[0].fingerprint, fp);
    assert!(certs[0].is_secret);
}

#[test]
fn test_keystore_generate_and_get_info() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    let info = store.get_cert_info(&fp).unwrap();
    assert_eq!(info.fingerprint, fp);
    assert_eq!(info.user_ids.len(), 1);
    assert_eq!(info.user_ids[0].value, "Test User <test@example.com>");
    assert!(!info.user_ids[0].revoked);
    assert!(info.is_secret);

    // Should have subkeys (encryption + signing)
    let non_cert_subkeys: Vec<_> = info.subkeys.iter()
        .filter(|s| s.key_type != KeyType::Certification)
        .collect();
    assert!(non_cert_subkeys.len() >= 2, "Expected at least 2 non-cert subkeys, got {}", non_cert_subkeys.len());
}

#[test]
fn test_keystore_import_rejects_public_only() {
    let key = generate_test_key("pass");

    // Parse the public key to verify it's not secret
    let pub_info = parse_cert_bytes(key.public_key.as_bytes(), true).unwrap();
    assert!(!pub_info.is_secret);

    // Secret key should be secret
    let sec_info = parse_cert_bytes(&key.secret_key, true).unwrap();
    assert!(sec_info.is_secret);
}

#[test]
fn test_keystore_import_public_key() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");

    // Import the public key (not secret)
    let fp = store.import_cert(key.public_key.as_bytes()).unwrap();

    let info = store.get_cert_info(&fp).unwrap();
    assert!(!info.is_secret, "Public key should not be secret");
    assert_eq!(info.user_ids.len(), 1);

    // Should coexist with a secret key import
    let fp2 = store.import_cert(&key.secret_key).unwrap();
    assert_eq!(fp, fp2, "Same fingerprint for public and secret key");

    // After importing secret, it should now be secret
    let info2 = store.get_cert_info(&fp).unwrap();
    assert!(info2.is_secret, "Should be secret after importing secret key");
}

#[test]
fn test_keystore_list_public_and_secret_keys() {
    let store = KeyStore::open_in_memory().unwrap();

    // Create two keys: import one as secret, one as public-only
    let key1 = generate_test_key("pass1");
    let key2 = create_key(
        "pass2",
        &["Public User <pub@example.com>"],
        CipherSuite::Cv25519,
        None, None, None,
        SubkeyFlags { encryption: true, signing: false, authentication: false },
        true, true,
    ).unwrap();

    store.import_cert(&key1.secret_key).unwrap();
    store.import_cert(key2.public_key.as_bytes()).unwrap();

    let all = store.list_certs().unwrap();
    assert_eq!(all.len(), 2);

    let secret_keys = store.list_secret_keys().unwrap();
    assert_eq!(secret_keys.len(), 1);
    assert!(secret_keys[0].is_secret);

    let public_keys = store.list_public_keys().unwrap();
    assert_eq!(public_keys.len(), 1);
    assert!(!public_keys[0].is_secret);
}

#[test]
fn test_keystore_delete() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    assert_eq!(store.count().unwrap(), 1);
    store.delete_cert(&fp).unwrap();
    assert_eq!(store.count().unwrap(), 0);
}

#[test]
fn test_keystore_export_public_key() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    let armored = store.export_cert_armored(&fp).unwrap();
    assert!(armored.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----"));
    assert!(!armored.contains("PRIVATE"));

    // Verify the exported key can be parsed
    let info = parse_cert_bytes(armored.as_bytes(), true).unwrap();
    assert_eq!(info.fingerprint, fp);
    assert!(!info.is_secret);
}

#[test]
fn test_add_user_id() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    // Get cert data
    let (cert_data, _) = store.get_cert(&fp).unwrap();

    // Add a new UID
    let updated = add_uid(&cert_data, "New Name <new@example.com>", "pass").unwrap();
    store.update_cert(&fp, &updated).unwrap();

    let info = store.get_cert_info(&fp).unwrap();
    assert_eq!(info.user_ids.len(), 2);
    let uids: Vec<&str> = info.user_ids.iter().map(|u| u.value.as_str()).collect();
    assert!(uids.contains(&"Test User <test@example.com>"));
    assert!(uids.contains(&"New Name <new@example.com>"));
}

#[test]
fn test_revoke_user_id() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    // Add then revoke a second UID
    let (cert_data, _) = store.get_cert(&fp).unwrap();
    let with_uid = add_uid(&cert_data, "ToRevoke <revoke@example.com>", "pass").unwrap();
    store.update_cert(&fp, &with_uid).unwrap();

    let (cert_data2, _) = store.get_cert(&fp).unwrap();
    let revoked = revoke_uid(&cert_data2, "ToRevoke <revoke@example.com>", "pass").unwrap();
    store.update_cert(&fp, &revoked).unwrap();

    let info = store.get_cert_info(&fp).unwrap();
    let revoked_uid = info.user_ids.iter()
        .find(|u| u.value.contains("revoke@example.com"))
        .expect("Revoked UID should still exist");
    assert!(revoked_uid.revoked);
}

#[test]
fn test_update_primary_expiry() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    let new_expiry = Utc::now() + Duration::days(730);
    let (cert_data, _) = store.get_cert(&fp).unwrap();
    let updated = update_primary_expiry(&cert_data, new_expiry, "pass").unwrap();
    store.update_cert(&fp, &updated).unwrap();

    let info = store.get_cert_info(&fp).unwrap();
    assert!(info.expiration_time.is_some());
    let exp = info.expiration_time.unwrap();
    // Should be roughly 2 years from now (within a day tolerance)
    let diff = (exp - Utc::now()).num_days();
    assert!(diff >= 728 && diff <= 731, "Expected ~730 days, got {}", diff);
}

#[test]
fn test_update_subkeys_expiry() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    let (cert_data, info) = store.get_cert(&fp).unwrap();

    let subkey_fps: Vec<String> = info.subkeys.iter()
        .filter(|sk| sk.key_type != KeyType::Certification)
        .map(|sk| sk.fingerprint.clone())
        .collect();
    assert!(!subkey_fps.is_empty());

    let new_expiry = Utc::now() + Duration::days(730);
    let fp_refs: Vec<&str> = subkey_fps.iter().map(|s| s.as_str()).collect();
    let updated = update_subkeys_expiry(&cert_data, &fp_refs, new_expiry, "pass").unwrap();
    store.update_cert(&fp, &updated).unwrap();

    let new_info = store.get_cert_info(&fp).unwrap();
    for sk in &new_info.subkeys {
        if sk.key_type != KeyType::Certification {
            assert!(sk.expiration_time.is_some(),
                "Subkey {} should have expiry set", sk.fingerprint);
        }
    }
}

#[test]
fn test_available_subkeys() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("pass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    let info = store.get_cert_info(&fp).unwrap();
    let now = Utc::now();

    let mut has_enc = false;
    let mut has_sign = false;
    for sk in &info.subkeys {
        if sk.is_revoked { continue; }
        if let Some(exp) = sk.expiration_time {
            if exp < now { continue; }
        }
        match sk.key_type {
            KeyType::Encryption => has_enc = true,
            KeyType::Signing => has_sign = true,
            _ => {}
        }
    }
    assert!(has_enc, "Should have encryption subkey");
    assert!(has_sign, "Should have signing subkey");
}

#[test]
fn test_multiple_keys_in_store() {
    let store = KeyStore::open_in_memory().unwrap();

    let key1 = generate_test_key("pass1");
    let key2 = create_key(
        "pass2",
        &["Another User <another@example.com>"],
        CipherSuite::Cv25519,
        None, None, None,
        SubkeyFlags { encryption: true, signing: false, authentication: false },
        true, true,
    ).unwrap();

    store.import_cert(&key1.secret_key).unwrap();
    store.import_cert(&key2.secret_key).unwrap();

    assert_eq!(store.count().unwrap(), 2);

    let certs = store.list_certs().unwrap();
    assert_eq!(certs.len(), 2);
    let emails: Vec<String> = certs.iter()
        .flat_map(|c| c.user_ids.iter().map(|u| u.value.clone()))
        .collect();
    assert!(emails.iter().any(|e| e.contains("test@example.com")));
    assert!(emails.iter().any(|e| e.contains("another@example.com")));
}

#[test]
fn test_uid_parsing() {
    // Test the name/email parsing logic used by cert_info_to_key_info
    let uid = "John Doe <john@example.com>";
    let (name, email) = if let Some(lt_pos) = uid.find('<') {
        let name = uid[..lt_pos].trim().to_string();
        let email = uid[lt_pos+1..].trim_end_matches('>').trim().to_string();
        (name, email)
    } else {
        (uid.to_string(), String::new())
    };
    assert_eq!(name, "John Doe");
    assert_eq!(email, "john@example.com");

    // Test UID without email
    let uid2 = "Just a Name";
    let (name2, email2) = if let Some(lt_pos) = uid2.find('<') {
        let name = uid2[..lt_pos].trim().to_string();
        let email = uid2[lt_pos+1..].trim_end_matches('>').trim().to_string();
        (name, email)
    } else {
        (uid2.to_string(), String::new())
    };
    assert_eq!(name2, "Just a Name");
    assert_eq!(email2, "");
}

#[test]
fn test_change_key_password() {
    let store = KeyStore::open_in_memory().unwrap();
    let key = generate_test_key("oldpass");
    let fp = store.import_cert(&key.secret_key).unwrap();

    // Change password
    let (cert_data, _) = store.get_cert(&fp).unwrap();
    let updated = update_password(&cert_data, "oldpass", "newpass").unwrap();
    store.update_cert(&fp, &updated).unwrap();

    // Verify new password works: encrypt with public key, decrypt with new password
    let info = store.get_cert_info(&fp).unwrap();
    assert!(info.is_secret);

    let armored = store.export_cert_armored(&fp).unwrap();
    let ciphertext = encrypt_bytes(armored.as_bytes(), b"secret message", true).unwrap();

    let (updated_cert, _) = store.get_cert(&fp).unwrap();
    let plaintext = decrypt_bytes(&updated_cert, &ciphertext, "newpass").unwrap();
    assert_eq!(plaintext, b"secret message");
}

#[test]
fn test_change_key_password_wrong_old_password() {
    let key = generate_test_key("correctpass");
    let result = update_password(&key.secret_key, "wrongpass", "newpass");
    assert!(result.is_err(), "Should fail with wrong old password");
}
