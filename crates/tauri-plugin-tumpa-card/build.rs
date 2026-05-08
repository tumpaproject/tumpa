// Tauri plugin build script — generates permission manifests and wires
// the Android Gradle module + iOS Swift package into the host app.
//
// Keep in sync with the `#[command]`s declared in `src/commands.rs`: each
// name that appears here must match a `pub fn` with a `#[tauri::command]`
// attribute on the Rust side (and a `@Command` on the native side).

const COMMANDS: &[&str] = &[
    "begin_session",
    "transmit_apdu",
    "end_session",
    // Keyring-backed secret storage (M6b). All reads go through a
    // biometric prompt; all writes require the device to have a
    // passcode set. Used for card PINs *and* on-disk key passphrases.
    "save_secret",
    "read_secret",
    "clear_secret",
    "clear_all_secrets",
];

fn main() {
    let result = tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .try_build();

    // Building docs for non-mobile targets trips this; the crate is only
    // meaningful on Android / iOS.
    if !(cfg!(docsrs) && std::env::var("TARGET").unwrap().contains("android")) {
        result.unwrap();
    }
}
