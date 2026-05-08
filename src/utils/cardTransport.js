// Tell the Rust-side card bridge which transport to use for the next
// card op. Mobile only; on desktop the PC/SC backend is wired up
// instead and this call is a no-op.
//
// Valid values: 'nfc', 'usb', 'auto'. `auto` lets the plugin pick
// (USB if a CCID reader is plugged in, NFC otherwise).
import { invoke } from '@tauri-apps/api/core'

export async function setCardTransport(transport) {
  try {
    await invoke('set_card_transport', { transport })
  } catch (e) {
    // Older apps / desktop builds may not register this command yet;
    // swallow the "not found" error so the caller UI keeps working.
    console.debug('set_card_transport not available:', e)
  }
}
