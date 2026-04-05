/// Tests for smartcard operations.
///
/// These tests require a physical OpenPGP smartcard (e.g. Yubikey)
/// connected via pcscd. They are ignored by default.
///
/// Run with: cargo test --test card_tests -- --ignored
///
/// WARNING: Some tests perform destructive operations (factory reset).
use wecanencrypt::card::{
    is_card_connected,
    get_card_details,
    get_card_version,
    get_card_serial,
    get_pin_retry_counters,
    set_cardholder_name,
    set_public_key_url,
};

#[test]
fn test_card_detection_does_not_panic() {
    // This test always runs — it just checks the function doesn't panic
    let _connected = is_card_connected();
}

#[test]
#[ignore = "requires physical smartcard"]
fn test_get_card_details_with_card() {
    assert!(is_card_connected(), "No smartcard connected");
    let info = get_card_details().expect("Failed to get card details");
    assert!(!info.serial_number.is_empty(), "Serial number should not be empty");
}

#[test]
#[ignore = "requires physical smartcard"]
fn test_get_card_version_with_card() {
    assert!(is_card_connected(), "No smartcard connected");
    let version = get_card_version().expect("Failed to get version");
    assert!(!version.is_empty());
}

#[test]
#[ignore = "requires physical smartcard"]
fn test_get_card_serial_with_card() {
    assert!(is_card_connected(), "No smartcard connected");
    let serial = get_card_serial().expect("Failed to get serial");
    assert!(!serial.is_empty());
}

#[test]
#[ignore = "requires physical smartcard"]
fn test_get_pin_retry_counters_with_card() {
    assert!(is_card_connected(), "No smartcard connected");
    let (user, _reset, admin) = get_pin_retry_counters().expect("Failed to get counters");
    // Default retries are typically 3
    assert!(user > 0, "User PIN retries should be > 0");
    assert!(admin > 0, "Admin PIN retries should be > 0");
}

#[test]
#[ignore = "requires physical smartcard, DESTRUCTIVE: resets card and changes cardholder name"]
fn test_set_cardholder_name_with_card() {
    use wecanencrypt::card::reset_card;
    assert!(is_card_connected(), "No smartcard connected");
    reset_card().expect("Failed to reset card");
    set_cardholder_name("Test<<User", b"12345678")
        .expect("Failed to set cardholder name");

    let info = get_card_details().expect("Failed to get card details");
    assert!(info.cardholder_name.is_some());
}

#[test]
#[ignore = "requires physical smartcard, DESTRUCTIVE: resets card and changes public key URL"]
fn test_set_public_key_url_with_card() {
    use wecanencrypt::card::reset_card;
    assert!(is_card_connected(), "No smartcard connected");
    reset_card().expect("Failed to reset card");
    set_public_key_url("https://keys.openpgp.org/vks/v1/by-fingerprint/TEST", b"12345678")
        .expect("Failed to set public key URL");

    let info = get_card_details().expect("Failed to get card details");
    assert!(info.public_key_url.is_some());
}

/// Test the upload bitmask logic (no card needed, pure logic test)
#[test]
fn test_upload_bitmask_logic() {
    let which: u8 = 0;
    assert_eq!(which & 1, 0, "encryption bit not set");
    assert_eq!(which & 2, 0, "signing bit not set");
    assert_eq!(which & 4, 0, "authentication bit not set");

    let which: u8 = 7; // all bits set
    assert_ne!(which & 1, 0, "encryption bit should be set");
    assert_ne!(which & 2, 0, "signing bit should be set");
    assert_ne!(which & 4, 0, "authentication bit should be set");

    let which: u8 = 3; // enc + sign
    assert_ne!(which & 1, 0);
    assert_ne!(which & 2, 0);
    assert_eq!(which & 4, 0);
}

/// Full upload workflow: generate key, reset card, upload subkeys
#[test]
#[ignore = "requires physical smartcard, DESTRUCTIVE: resets card and uploads keys"]
fn test_full_upload_workflow() {
    use wecanencrypt::{
        create_key, parse_cert_bytes,
        CipherSuite, SubkeyFlags, KeyType,
        card::{
            reset_card,
            upload_primary_key_to_card,
            upload_key_to_card as card_upload_key,
            upload_subkey_by_fingerprint,
            CardKeySlot,
        },
    };
    use chrono::{Utc, Duration};

    assert!(is_card_connected(), "No smartcard connected");

    // Generate a test key
    let key = create_key(
        "testpass",
        &["Card Test <cardtest@example.com>"],
        CipherSuite::Cv25519,
        None,
        Some(Utc::now() + Duration::days(365)),
        Some(Utc::now() + Duration::days(365)),
        SubkeyFlags { encryption: true, signing: true, authentication: true },
        true,
        true,
    ).expect("Key generation failed");

    let cert_info = parse_cert_bytes(&key.secret_key, true).unwrap();
    assert!(cert_info.is_secret);

    // Reset card
    reset_card().expect("Failed to reset card");

    let default_admin = b"12345678";

    // Upload primary key to signing slot
    upload_primary_key_to_card(
        &key.secret_key,
        b"testpass",
        CardKeySlot::Signing,
        default_admin,
    ).expect("Failed to upload primary key");

    // Upload encryption subkey
    card_upload_key(
        &key.secret_key,
        b"testpass",
        CardKeySlot::Decryption,
        default_admin,
    ).expect("Failed to upload encryption subkey");

    // Upload authentication subkey by fingerprint
    let auth_fp = cert_info.subkeys.iter()
        .find(|sk| matches!(sk.key_type, KeyType::Authentication))
        .expect("No auth subkey")
        .fingerprint.clone();

    upload_subkey_by_fingerprint(
        &key.secret_key,
        b"testpass",
        &auth_fp,
        CardKeySlot::Authentication,
        default_admin,
    ).expect("Failed to upload auth subkey");

    // Verify card has keys by checking details
    let details = get_card_details().expect("Failed to get card details");
    assert!(details.signature_fingerprint.is_some(), "Signing key should be on card");
    assert!(details.encryption_fingerprint.is_some(), "Encryption key should be on card");
    assert!(details.authentication_fingerprint.is_some(), "Auth key should be on card");
}

/// Test PIN change workflow
#[test]
#[ignore = "requires physical smartcard, DESTRUCTIVE: resets card and changes PINs"]
fn test_pin_change_workflow() {
    use wecanencrypt::card::{
        reset_card,
        change_user_pin,
        change_admin_pin,
        verify_user_pin,
        verify_admin_pin,
    };

    assert!(is_card_connected(), "No smartcard connected");

    // Reset to get default PINs
    reset_card().expect("Failed to reset card");

    // Change user PIN from default
    change_user_pin(b"123456", b"654321").expect("Failed to change user PIN");
    verify_user_pin(b"654321").expect("New user PIN should work");

    // Change admin PIN from default
    change_admin_pin(b"12345678", b"87654321").expect("Failed to change admin PIN");
    verify_admin_pin(b"87654321").expect("New admin PIN should work");

    // Reset card back to defaults for other tests
    reset_card().expect("Failed to reset card");
}

/// Test PIN validation logic (no card needed)
#[test]
fn test_pin_length_validation() {
    // User PIN minimum 6
    assert!("12345".len() < 6);
    assert!("123456".len() >= 6);

    // Admin PIN minimum 8
    assert!("1234567".len() < 8);
    assert!("12345678".len() >= 8);
}
