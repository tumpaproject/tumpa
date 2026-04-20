// Thin JS wrapper around `tauri-plugin-tumpa-card`'s keyring IPC.
//
// Only meaningful on mobile. On desktop these calls will fail — callers
// should guard with `store.isMobile` before invoking.
//
// Identifier scheme (must match any other caller in the project):
//
//   card.pin.user          — OpenPGP card user PIN (PW1)
//   card.pin.admin         — OpenPGP card admin PIN (PW3)
//   key.pass.<FINGERPRINT> — passphrase for the on-disk secret key
//                            with the given uppercase fingerprint
//
// Every read triggers a biometric prompt. Saves are allowed freely
// within the app process; the OS still requires a device passcode.

import { invoke } from '@tauri-apps/api/core'

export function keyForUserPin() { return 'card.pin.user' }
export function keyForAdminPin() { return 'card.pin.admin' }
export function keyForKeyPassphrase(fingerprint) {
  return `key.pass.${String(fingerprint).toUpperCase()}`
}

/// Save a secret (bytes or string) to the platform keyring.
export async function saveSecret(key, secret) {
  const bytes = typeof secret === 'string'
    ? Array.from(new TextEncoder().encode(secret))
    : Array.from(secret)
  await invoke('plugin:tumpa-card|save_secret', {
    req: { key, secret: bytes },
  })
}

/// Read a secret. Always presents a biometric prompt with `reason`
/// shown to the user. Returns the bytes as Uint8Array.
///
/// Rejects with:
/// - "no saved secret under that key" if nothing is stored there
/// - "cancelled by user" if the user dismisses the biometric sheet
///
/// Callers typically want to catch the first one (fall back to
/// prompting) and surface the second as a gentle "Try again".
export async function readSecret(key, reason) {
  const resp = await invoke('plugin:tumpa-card|read_secret', {
    req: { key, reason },
  })
  return new Uint8Array(resp.secret)
}

/// Like `readSecret` but returns the result as a UTF-8 string (for
/// passphrases typed as text). Callers still need to zeroize / null
/// out their reference after use — we can't enforce that from JS.
export async function readSecretAsString(key, reason) {
  const bytes = await readSecret(key, reason)
  return new TextDecoder().decode(bytes)
}

export async function clearSecret(key) {
  await invoke('plugin:tumpa-card|clear_secret', {
    req: { key },
  })
}

export async function clearAllSecrets() {
  await invoke('plugin:tumpa-card|clear_all_secrets')
}

/// True if the error string looks like the "no saved secret under
/// that key" signal from the plugin. Useful for silently falling
/// back to a text-entry prompt.
export function isMissingSecretError(err) {
  return String(err).toLowerCase().includes('no saved secret')
}

/// True if the user dismissed the biometric / OS prompt.
export function isCancelledError(err) {
  return String(err).toLowerCase().includes('cancel')
}
