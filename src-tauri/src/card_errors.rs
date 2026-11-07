//! Turn low-level card errors into messages that name the PIN involved.
//!
//! wecanencrypt maps status word 6983 ("authentication method blocked")
//! to the generic `"PIN is blocked"`, without saying *which* PIN. The
//! UI then shows that text as is, and the user cannot tell the two PINs apart.
//!
//! Kept free of Tauri / card types so it can be unit-tested from
//! `src-tauri/tests/` without hardware.

/// Shown when the admin PIN (PW3) has no retries left.
pub const ADMIN_PIN_BLOCKED_MSG: &str = "The admin PIN is blocked (no attempts left), so it cannot \
     be used to set a new user PIN. A blocked admin PIN can only be recovered by factory \
     resetting the card, which erases the keys on it.";

/// Rewrite an error from the admin-authorized "change user PIN" flow.
///
/// That flow sends `VERIFY PW3` followed by `RESET RETRY COUNTER (PW1)`.
/// `RESET RETRY COUNTER` with P1=02 works even when PW1 is blocked, so
/// the only command in the flow that can answer 6983 is `VERIFY PW3`.
/// In other words, a "blocked" error here always means the **admin**
/// PIN is blocked, never the user PIN.
pub fn explain_change_user_pin_error(msg: String) -> String {
    let lower = msg.to_ascii_lowercase();
    if lower.contains("blocked") || lower.contains("6983") {
        ADMIN_PIN_BLOCKED_MSG.to_string()
    } else {
        msg
    }
}
