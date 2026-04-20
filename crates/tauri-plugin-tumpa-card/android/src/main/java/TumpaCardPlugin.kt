// SPDX-License-Identifier: GPL-3.0-or-later
//
// Tauri mobile plugin — OpenPGP smartcard bridge.
//
// Android transports:
//
// - **NFC** via `IsoDep.transceive` — implemented. Vendor-neutral:
//   any ISO-DEP compliant contactless smartcard (YubiKey 5 NFC,
//   Nitrokey 3 NFC, Gnuk, …) works.
//
// - **USB-C** via UsbManager + CCID — stubbed. Needs a minimal CCID
//   implementation in a follow-up. Rejects with a clear message.
//
// Keyring (M6b): stubbed. iOS has a real Keychain impl; the Android
// equivalent (EncryptedSharedPreferences + BiometricPrompt) is
// pending.
//
// ### Session lifetime — IMPORTANT
//
// libtumpa's higher-level flows (e.g. `upload`) call several
// wecanencrypt functions in sequence (`reset_card`,
// `upload_primary_key_to_card`, `upload_subkey_by_fingerprint`, …).
// Each of those creates a fresh `MobileCardBackend` on the Rust side,
// which calls `beginSession` on entry and `endSession` on drop.
//
// If we actually opened a new NFC session per call, the user would
// have to tap their card 3–5 times for a single upload — unusable.
//
// So this plugin keeps the `IsoDep` alive between `endSession` and
// the next `beginSession`. As long as the user keeps the card
// pressed against the phone, all the internal ops reuse the same
// hardware connection (Android's presence-check loop keeps the tag
// activated in the background). If the card is lifted, the IsoDep
// silently invalidates; the next `beginSession` then has to arm
// reader mode and wait for a new tap.
//
// The logical `sessionId` changes on every `beginSession` so the
// Rust side's MobileCardBackend can track its own lifecycle, but all
// session IDs that exist during a given tap point at the same
// underlying IsoDep.

package `in`.kushaldas.tumpa.card

import android.app.Activity
import android.nfc.NfcAdapter
import android.nfc.Tag
import android.nfc.tech.IsoDep
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

@InvokeArg
class BeginSessionArgs {
    lateinit var transport: String        // "nfc" | "usb"
    lateinit var appletAid: IntArray
}

@InvokeArg
class TransmitApduArgs {
    lateinit var sessionId: String
    lateinit var apdu: IntArray
}

@InvokeArg
class EndSessionArgs {
    lateinit var sessionId: String
}

@InvokeArg
class SaveSecretArgs {
    lateinit var key: String
    lateinit var secret: IntArray
}

@InvokeArg
class ReadSecretArgs {
    lateinit var key: String
    lateinit var reason: String
}

@InvokeArg
class ClearSecretArgs {
    lateinit var key: String
}

private const val TAG = "TumpaCard"

@TauriPlugin
class TumpaCardPlugin(private val activity: Activity) : Plugin(activity) {

    /// Active Rust-side sessions. All values point at [activeIsoDep]
    /// while a physical tag is present. New entries are minted on each
    /// `beginSession`; entries are removed on `endSession` but the
    /// IsoDep itself isn't closed — see module comment.
    private val sessions = ConcurrentHashMap<String, IsoDep>()

    /// The one IsoDep we currently hold. Set when a tag is first
    /// discovered, cleared when the tag is removed (detected via
    /// transceive IOException) or when the UI explicitly finalizes.
    @Volatile
    private var activeIsoDep: IsoDep? = null

    @Volatile
    private var pendingBegin: PendingBegin? = null

    private data class PendingBegin(
        val invoke: Invoke,
        val aid: ByteArray,
    )

    private val nfcAdapter: NfcAdapter? by lazy {
        NfcAdapter.getDefaultAdapter(activity)
    }

    // -- APDU bridge -------------------------------------------------

    @Command
    fun beginSession(invoke: Invoke) {
        val args = invoke.parseArgs(BeginSessionArgs::class.java)
        val aid = intArrayToBytes(args.appletAid)
        when (args.transport) {
            "nfc" -> beginNfcSession(aid, invoke)
            "usb" -> invoke.reject(
                "USB CCID bridge not yet implemented on Android. Use NFC for now.")
            else -> invoke.reject("unknown transport: ${args.transport}")
        }
    }

    private fun beginNfcSession(aid: ByteArray, invoke: Invoke) {
        val adapter = nfcAdapter
        if (adapter == null) {
            invoke.reject("This device has no NFC hardware")
            return
        }
        if (!adapter.isEnabled) {
            invoke.reject("NFC is turned off in system settings")
            return
        }

        // Fast path — reuse the active IsoDep from a prior tap.
        // This is what turns a 3-5-tap wecanencrypt flow into a
        // single-tap UX.
        val current = activeIsoDep
        if (current != null && current.isConnected) {
            val sessionId = UUID.randomUUID().toString()
            sessions[sessionId] = current

            // Re-SELECT the OpenPGP applet. The card may have had
            // another applet selected by a prior operation, or the
            // transport stack may have reset selection during a
            // presence-check reconnect. Cheap idempotent guard.
            try {
                val selectResp = current.transceive(buildSelectApdu(aid))
                val sw1 = selectResp[selectResp.size - 2].toInt() and 0xff
                val sw2 = selectResp[selectResp.size - 1].toInt() and 0xff
                if (sw1 != 0x90 || sw2 != 0x00) {
                    sessions.remove(sessionId)
                    invoke.reject("SELECT failed ${"%02X%02X".format(sw1, sw2)} (reusing session)")
                    return
                }
            } catch (e: Exception) {
                Log.d(TAG, "reuse path: transceive failed, invalidating active IsoDep", e)
                sessions.remove(sessionId)
                try { current.close() } catch (_: Exception) {}
                activeIsoDep = null
                // Fall through to a full reader-mode arm so the user
                // can re-tap.
                armReaderMode(aid, invoke)
                return
            }

            Log.d(TAG, "beginSession: reused active IsoDep, sessionId=$sessionId")
            invoke.resolve(JSObject().apply {
                put("sessionId", sessionId)
                put("atr", bytesToJSArray(current.historicalBytes ?: ByteArray(0)))
            })
            return
        }

        // Slow path — no active tag, arm reader mode and wait.
        armReaderMode(aid, invoke)
    }

    private fun armReaderMode(aid: ByteArray, invoke: Invoke) {
        if (pendingBegin != null) {
            invoke.reject("another card session is already pending")
            return
        }
        pendingBegin = PendingBegin(invoke, aid)

        val flags = NfcAdapter.FLAG_READER_NFC_A or
            NfcAdapter.FLAG_READER_NFC_B or
            NfcAdapter.FLAG_READER_SKIP_NDEF_CHECK
        val extras = android.os.Bundle().apply {
            // 5 s presence check — gives the user time between ops
            // without the tag dropping, and long enough that a slow
            // card operation doesn't trigger a spurious reconnect.
            putInt(NfcAdapter.EXTRA_READER_PRESENCE_CHECK_DELAY, 5000)
        }
        nfcAdapter?.enableReaderMode(
            activity, { tag -> onTagDiscovered(tag) }, flags, extras
        )
        Log.d(TAG, "beginSession: reader mode armed, awaiting tap")
    }

    /// Background-thread callback from NFC subsystem.
    private fun onTagDiscovered(tag: Tag) {
        val pending = pendingBegin ?: return

        // Drop duplicate taps while a session is active. Reader mode
        // stays armed throughout a multi-APDU flow, so stray taps
        // from the user repositioning the card can otherwise hijack
        // the session.
        if (activeIsoDep != null) {
            Log.d(TAG, "onTagDiscovered: ignoring — session already active")
            return
        }

        val isoDep = IsoDep.get(tag)
        if (isoDep == null) {
            // Not ISO-DEP — ignore, let the user try a different
            // card. Reader mode stays armed.
            Log.d(TAG, "onTagDiscovered: non-ISO-DEP tag, waiting for another")
            return
        }

        try {
            isoDep.connect()
            isoDep.timeout = 15000
            Log.d(TAG, "onTagDiscovered: connected, aid=${pending.aid.toHexString()}")

            val selectCmd = buildSelectApdu(pending.aid)
            Log.d(TAG, "onTagDiscovered: SELECT cmd=${selectCmd.toHexString()}")
            val selectResp = isoDep.transceive(selectCmd)
            Log.d(TAG, "onTagDiscovered: SELECT resp=${selectResp.toHexString()}")

            if (selectResp.size < 2) {
                throw IllegalStateException("short SELECT response (${selectResp.size} bytes)")
            }
            val sw1 = selectResp[selectResp.size - 2].toInt() and 0xff
            val sw2 = selectResp[selectResp.size - 1].toInt() and 0xff
            if (sw1 != 0x90 || sw2 != 0x00) {
                val sw = "%02X%02X".format(sw1, sw2)
                Log.w(TAG, "onTagDiscovered: SELECT failed sw=$sw (card may not support OpenPGP)")
                try { isoDep.close() } catch (_: Exception) {}
                // Leave reader mode armed so the user can retry with
                // a different card. Only disarm when we succeed or
                // the session is explicitly finalized.
                pending.invoke.reject("SELECT failed $sw (card may not support OpenPGP)")
                pendingBegin = null
                return
            }

            // Success — keep the IsoDep alive and resolve.
            //
            // We deliberately do NOT call `disableReaderMode(activity)`
            // here. Android's NFC stack disconnects the currently
            // connected tag when reader mode is disabled, which shows
            // up as `TagLostException` on the very next transceive
            // (observed in the field: SELECT succeeded, disable call,
            // first real APDU 10ms later → TagLost). Reader mode stays
            // armed for the whole multi-APDU flow; onTagDiscovered
            // bails out if activeIsoDep is already set so a second
            // tap can't hijack the session.
            val sessionId = UUID.randomUUID().toString()
            activeIsoDep = isoDep
            sessions[sessionId] = isoDep
            pendingBegin = null

            Log.d(TAG, "onTagDiscovered: SELECT ok, sessionId=$sessionId")
            pending.invoke.resolve(JSObject().apply {
                put("sessionId", sessionId)
                put("atr", bytesToJSArray(isoDep.historicalBytes ?: ByteArray(0)))
            })

            // Tell the UI the card is now in hand so it can switch
            // the overlay from "Tap your card" to "Working — keep
            // the card pressed". We trigger only on the slow path
            // (actual tap); reused-session begins don't need this
            // since no tap happened.
            trigger("card-connected", JSObject())
        } catch (e: Exception) {
            try { isoDep.close() } catch (_: Exception) {}
            nfcAdapter?.disableReaderMode(activity)
            pending.invoke.reject(e.message ?: "NFC error during SELECT")
            pendingBegin = null
        }
    }

    @Command
    fun transmitApdu(invoke: Invoke) {
        val args = invoke.parseArgs(TransmitApduArgs::class.java)
        val isoDep = sessions[args.sessionId]
        if (isoDep == null) {
            invoke.reject("no-active-session")
            return
        }
        try {
            val response = isoDep.transceive(intArrayToBytes(args.apdu))
            invoke.resolve(JSObject().apply {
                put("response", bytesToJSArray(response))
            })
        } catch (e: Exception) {
            // Transceive failure almost always means the tag fell
            // off. Invalidate activeIsoDep so the next beginSession
            // re-arms reader mode.
            Log.d(TAG, "transmitApdu: failed (tag likely lost)", e)
            try { isoDep.close() } catch (_: Exception) {}
            sessions.values.removeIf { it === isoDep }
            if (activeIsoDep === isoDep) activeIsoDep = null
            invoke.reject(e.message ?: "transmit failed")
        }
    }

    @Command
    fun endSession(invoke: Invoke) {
        val args = invoke.parseArgs(EndSessionArgs::class.java)
        // Remove the session id but DO NOT close the IsoDep — the
        // next beginSession in this flow reuses it. If the user has
        // already lifted the card, the next transceive will fail and
        // trigger the invalidation path in transmitApdu.
        sessions.remove(args.sessionId)
        Log.d(TAG, "endSession: sessionId=${args.sessionId} (IsoDep kept alive)")
        invoke.resolve(JSObject())
    }

    // -- Keyring-backed secret storage (M6b) — Android TODO -----------

    @Command
    fun saveSecret(invoke: Invoke) {
        val args = invoke.parseArgs(SaveSecretArgs::class.java)
        invoke.reject("tumpa-card keyring not yet implemented on Android (saveSecret key=${args.key})")
    }

    @Command
    fun readSecret(invoke: Invoke) {
        val args = invoke.parseArgs(ReadSecretArgs::class.java)
        invoke.reject("tumpa-card keyring not yet implemented on Android (readSecret key=${args.key}, reason=${args.reason})")
    }

    @Command
    fun clearSecret(invoke: Invoke) {
        val args = invoke.parseArgs(ClearSecretArgs::class.java)
        invoke.reject("tumpa-card keyring not yet implemented on Android (clearSecret key=${args.key})")
    }

    @Command
    fun clearAllSecrets(invoke: Invoke) {
        invoke.reject("tumpa-card keyring not yet implemented on Android (clearAllSecrets)")
    }

    // -- helpers ------------------------------------------------------

    private fun intArrayToBytes(arr: IntArray): ByteArray {
        val out = ByteArray(arr.size)
        for (i in arr.indices) {
            out[i] = (arr[i] and 0xff).toByte()
        }
        return out
    }

    private fun bytesToJSArray(bytes: ByteArray): JSArray {
        val arr = JSArray()
        for (b in bytes) {
            arr.put(b.toInt() and 0xff)
        }
        return arr
    }

    /// Hex-encode bytes for log messages. Not cryptographically
    /// relevant — purely a debugging aid to see APDUs and SW codes.
    private fun ByteArray.toHexString(): String =
        joinToString("") { "%02X".format(it.toInt() and 0xff) }

    private fun buildSelectApdu(aid: ByteArray): ByteArray {
        // ISO 7816-4 SELECT by name: CLA INS P1 P2 Lc [AID] Le
        val out = ByteArray(6 + aid.size)
        out[0] = 0x00
        out[1] = 0xA4.toByte()
        out[2] = 0x04
        out[3] = 0x00
        out[4] = aid.size.toByte()
        System.arraycopy(aid, 0, out, 5, aid.size)
        out[5 + aid.size] = 0x00
        return out
    }
}
