// SPDX-License-Identifier: GPL-3.0-or-later
//
// Tauri mobile plugin — OpenPGP smartcard bridge.
//
// Android transports:
//
// - **NFC** via `IsoDep.transceive` — any ISO-DEP compliant contactless
//   smartcard (YubiKey 5 NFC, Nitrokey 3 NFC, Gnuk, …).
// - **USB-C** via `UsbManager` + CCID — see UsbCcidConnection.kt.
//   Vendor-neutral: any USB class 0x0B reader.
// - **Auto**: picks USB if a CCID reader is already plugged in,
//   otherwise NFC. This is what the Rust `card_bridge` ships with so
//   users never have to think about which transport they're using.
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
// On USB the issue is subtler but the same: power-cycling the card
// on every end/begin resets OpenPGP applet state.
//
// So this plugin keeps the physical handle (IsoDep or UsbCcidConnection)
// alive between `endSession` and the next `beginSession`. As long as
// the user keeps the card pressed / plugged in, all the internal ops
// reuse the same hardware connection. If the card is lifted / cable
// yanked, the next transceive fails and the handle is invalidated;
// the next `beginSession` then has to arm reader mode / re-enumerate.
//
// The logical `sessionId` changes on every `beginSession` so the
// Rust side's MobileCardBackend can track its own lifecycle, but all
// session IDs that exist during a given tap / plug-in point at the
// same underlying physical handle.

package `in`.kushaldas.tumpa.card

import android.app.Activity
import android.content.Context
import android.hardware.usb.UsbManager
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
import java.util.concurrent.Executors

@InvokeArg
class BeginSessionArgs {
    lateinit var transport: String        // "nfc" | "usb" | "auto"
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

/// Physical handle backing a logical session. The plugin tracks one
/// sealed-class instance per Rust-side session id; multiple session
/// ids created during a single hardware interaction share the same
/// handle.
private sealed class SessionHandle {
    class Nfc(val isoDep: IsoDep) : SessionHandle()
    class Usb(val connection: UsbCcidConnection) : SessionHandle()
}

@TauriPlugin
class TumpaCardPlugin(private val activity: Activity) : Plugin(activity) {

    /// Active Rust-side sessions keyed by the UUID we return from
    /// beginSession. Values may alias — a single physical tap/plug-in
    /// backs multiple logical sessions across libtumpa's sequential
    /// wecanencrypt calls.
    private val sessions = ConcurrentHashMap<String, SessionHandle>()

    /// The one IsoDep we currently hold. Set when a tag is first
    /// discovered, cleared when the tag is removed (detected via
    /// transceive IOException) or when the UI explicitly finalizes.
    @Volatile
    private var activeIsoDep: IsoDep? = null

    /// The one USB CCID connection we currently hold. Persists across
    /// end/begin cycles so a multi-APDU flow reuses the same power-on.
    @Volatile
    private var activeUsb: UsbCcidConnection? = null

    @Volatile
    private var pendingBegin: PendingBegin? = null

    private data class PendingBegin(
        val invoke: Invoke,
        val aid: ByteArray,
    )

    private val nfcAdapter: NfcAdapter? by lazy {
        NfcAdapter.getDefaultAdapter(activity)
    }

    private val usbManager: UsbManager by lazy {
        activity.getSystemService(Context.USB_SERVICE) as UsbManager
    }

    private val usbPermission by lazy { UsbPermissionRequester(activity) }

    /// Dedicated background executor for USB CCID work. The permission
    /// callback fires on the main thread; we must not do bulk reads
    /// there or the 10s BroadcastReceiver limit — and the 5s ANR
    /// input limit for long-running card ops — will kill the process.
    private val usbExecutor = Executors.newSingleThreadExecutor()

    // -- APDU bridge -------------------------------------------------

    @Command
    fun beginSession(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(BeginSessionArgs::class.java)
            val aid = intArrayToBytes(args.appletAid)
            Log.d(TAG, "beginSession: transport=${args.transport} aid=${aid.toHexString()}")
            beginSessionInner(args.transport, aid, invoke)
        } catch (e: Throwable) {
            Log.w(TAG, "beginSession: uncaught", e)
            try { invoke.reject(e.message ?: e.javaClass.simpleName) } catch (_: Exception) {}
        }
    }

    private fun beginSessionInner(transport: String, aid: ByteArray, invoke: Invoke) {
        when (transport) {
            "nfc" -> beginNfcSession(aid, invoke)
            "usb" -> beginUsbSession(aid, invoke, requireDevice = true)
            "auto" -> {
                // Always check current device presence — a warm
                // `activeUsb` stays `isOpen=true` until the next failing
                // transmit, so if the user unplugged the cable after a
                // prior flow we'd keep trying USB and NFC would never
                // work. Trust the USB device list as the source of truth.
                val usbDevice = UsbCcidConnection.findAttachedCcidDevice(usbManager)
                if (usbDevice != null) {
                    beginUsbSession(aid, invoke, requireDevice = true)
                    return
                }
                // No reader attached — drop any stale USB handle so NFC
                // paths aren't confused. Do NOT call close() here: the
                // handle is already dead (cable unplugged), and
                // close() tries an IccPowerOff via bulkTransfer which
                // can block for the full 30s BULK_TIMEOUT_MS on a
                // disconnected device. Nulling the handle is enough
                // for GC to clean up.
                activeUsb?.let {
                    activeUsb = null
                    sessions.values.removeIf { it is SessionHandle.Usb }
                }
                beginNfcSession(aid, invoke)
            }
            else -> invoke.reject("unknown transport: $transport")
        }
    }

    // -- NFC ---------------------------------------------------------

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
        //
        // `IsoDep.isConnected` calls through to the NFC service; once
        // the tag has physically left the field Android invalidates the
        // Tag object and the next `isConnected` call throws
        // `SecurityException: Tag is out of date`. Treat that (and any
        // other exception) as "tag gone" — drop the handle, fall
        // through to reader-mode arm, and let the user re-tap.
        val current = activeIsoDep
        val stillConnected = try {
            current != null && current.isConnected
        } catch (e: Exception) {
            Log.d(TAG, "beginNfcSession: stale IsoDep probe failed (${e.javaClass.simpleName}: ${e.message})")
            activeIsoDep = null
            sessions.values.removeIf { it is SessionHandle.Nfc }
            false
        }
        Log.d(TAG, "beginNfcSession: activeIsoDep=${current != null} stillConnected=$stillConnected")

        if (current != null && stillConnected) {
            val sessionId = UUID.randomUUID().toString()
            sessions[sessionId] = SessionHandle.Nfc(current)

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

        armReaderMode(aid, invoke)
    }

    private fun armReaderMode(aid: ByteArray, invoke: Invoke) {
        // If a previous beginSession is still armed (user backed out
        // of the overlay without tapping, or closed+reopened the
        // screen), reject the old pending invoke and take over. The
        // old invoke belongs to a view that has already moved on; its
        // caller won't mind the rejection and the UI needs a fresh
        // arm to respond to the new tap.
        pendingBegin?.let { stale ->
            Log.d(TAG, "armReaderMode: superseding stale pendingBegin")
            try { stale.invoke.reject("superseded by newer begin_session") } catch (_: Exception) {}
            try { nfcAdapter?.disableReaderMode(activity) } catch (_: Exception) {}
            pendingBegin = null
        }
        pendingBegin = PendingBegin(invoke, aid)

        val flags = NfcAdapter.FLAG_READER_NFC_A or
            NfcAdapter.FLAG_READER_NFC_B or
            NfcAdapter.FLAG_READER_SKIP_NDEF_CHECK
        val extras = android.os.Bundle().apply {
            putInt(NfcAdapter.EXTRA_READER_PRESENCE_CHECK_DELAY, 5000)
        }
        nfcAdapter?.enableReaderMode(
            activity, { tag -> onTagDiscovered(tag) }, flags, extras
        )
        Log.d(TAG, "beginSession: reader mode armed, awaiting tap")
    }

    /// Explicit cancel — called by the UI when the user taps Cancel
    /// on the overlay. Aborts any pending NFC reader mode and rejects
    /// the in-flight begin_session, so the Rust-side MobileCardBackend
    /// drops cleanly and the UI's `invoke()` promise settles.
    @Command
    fun cancelSession(invoke: Invoke) {
        pendingBegin?.let { stale ->
            Log.d(TAG, "cancelSession: rejecting pending begin")
            try { stale.invoke.reject("cancelled") } catch (_: Exception) {}
            try { nfcAdapter?.disableReaderMode(activity) } catch (_: Exception) {}
            pendingBegin = null
        }
        invoke.resolve(JSObject())
    }

    /// Background-thread callback from NFC subsystem.
    private fun onTagDiscovered(tag: Tag) {
        val pending = pendingBegin ?: return

        if (activeIsoDep != null) {
            Log.d(TAG, "onTagDiscovered: ignoring — session already active")
            return
        }

        val isoDep = IsoDep.get(tag)
        if (isoDep == null) {
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
                pending.invoke.reject("SELECT failed $sw (card may not support OpenPGP)")
                pendingBegin = null
                return
            }

            val sessionId = UUID.randomUUID().toString()
            activeIsoDep = isoDep
            sessions[sessionId] = SessionHandle.Nfc(isoDep)
            pendingBegin = null

            Log.d(TAG, "onTagDiscovered: SELECT ok, sessionId=$sessionId")
            pending.invoke.resolve(JSObject().apply {
                put("sessionId", sessionId)
                put("atr", bytesToJSArray(isoDep.historicalBytes ?: ByteArray(0)))
            })

            trigger("card-connected", JSObject())
        } catch (e: Exception) {
            try { isoDep.close() } catch (_: Exception) {}
            nfcAdapter?.disableReaderMode(activity)
            pending.invoke.reject(e.message ?: "NFC error during SELECT")
            pendingBegin = null
        }
    }

    // -- USB ---------------------------------------------------------

    private fun beginUsbSession(aid: ByteArray, invoke: Invoke, requireDevice: Boolean) {
        // Fast path — reuse the active USB connection from a prior call.
        val current = activeUsb
        if (current != null && current.isOpen) {
            val sessionId = UUID.randomUUID().toString()
            sessions[sessionId] = SessionHandle.Usb(current)

            try {
                current.rememberSelectedAid(aid)
                val selectResp = current.transmit(buildSelectApdu(aid))
                if (selectResp.size < 2) {
                    throw IllegalStateException("short SELECT response")
                }
                val sw1 = selectResp[selectResp.size - 2].toInt() and 0xff
                val sw2 = selectResp[selectResp.size - 1].toInt() and 0xff
                if (sw1 != 0x90 || sw2 != 0x00) {
                    sessions.remove(sessionId)
                    invoke.reject("SELECT failed ${"%02X%02X".format(sw1, sw2)} (reusing USB session)")
                    return
                }
            } catch (e: Exception) {
                Log.d(TAG, "usb reuse: transmit failed, invalidating", e)
                sessions.remove(sessionId)
                try { current.close() } catch (_: Exception) {}
                activeUsb = null
                // Fall through to full enumeration.
                enumerateAndBeginUsb(aid, invoke, requireDevice)
                return
            }

            Log.d(TAG, "beginUsbSession: reused active USB, sessionId=$sessionId")
            invoke.resolve(JSObject().apply {
                put("sessionId", sessionId)
                put("atr", bytesToJSArray(current.atr))
            })
            return
        }

        enumerateAndBeginUsb(aid, invoke, requireDevice)
    }

    private fun enumerateAndBeginUsb(aid: ByteArray, invoke: Invoke, requireDevice: Boolean) {
        val device = UsbCcidConnection.findAttachedCcidDevice(usbManager)
        if (device == null) {
            val msg = "no CCID smartcard reader attached over USB"
            if (requireDevice) invoke.reject(msg) else invoke.reject(msg)
            return
        }
        Log.d(TAG, "usb: requesting permission for ${device.deviceName} (VID=${"%04x".format(device.vendorId)} PID=${"%04x".format(device.productId)})")
        usbPermission.request(usbManager, device) { granted ->
            Log.d(TAG, "usb permission callback: granted=$granted")
            // The permission callback fires on the main thread. Hop to
            // the USB executor before any bulk I/O so we don't ANR.
            usbExecutor.execute {
                Log.d(TAG, "usbExecutor: entering")
                if (!granted) {
                    invoke.reject("USB permission denied")
                    return@execute
                }
                try {
                    Log.d(TAG, "usbExecutor: opening CCID connection")
                    val conn = UsbCcidConnection.open(usbManager, device)
                    Log.d(TAG, "usbExecutor: opened, sending SELECT")
                    conn.rememberSelectedAid(aid)
                    val selectResp = conn.transmit(buildSelectApdu(aid))
                    if (selectResp.size < 2) {
                        conn.close()
                        invoke.reject("short SELECT response on USB")
                        return@execute
                    }
                    val sw1 = selectResp[selectResp.size - 2].toInt() and 0xff
                    val sw2 = selectResp[selectResp.size - 1].toInt() and 0xff
                    if (sw1 != 0x90 || sw2 != 0x00) {
                        val sw = "%02X%02X".format(sw1, sw2)
                        conn.close()
                        invoke.reject("SELECT failed $sw (card may not support OpenPGP)")
                        return@execute
                    }
                    val sessionId = UUID.randomUUID().toString()
                    activeUsb = conn
                    sessions[sessionId] = SessionHandle.Usb(conn)
                    Log.d(TAG, "beginUsbSession: SELECT ok, sessionId=$sessionId, ATR=${conn.atr.toHexString()}")
                    invoke.resolve(JSObject().apply {
                        put("sessionId", sessionId)
                        put("atr", bytesToJSArray(conn.atr))
                    })
                    trigger("card-connected", JSObject())
                } catch (e: Exception) {
                    Log.w(TAG, "beginUsbSession: open/select failed", e)
                    invoke.reject(e.message ?: "USB CCID setup failed")
                }
            }
        }
    }

    // -- Transmit / End ----------------------------------------------

    @Command
    fun transmitApdu(invoke: Invoke) {
        val args = invoke.parseArgs(TransmitApduArgs::class.java)
        val handle = sessions[args.sessionId]
        if (handle == null) {
            invoke.reject("no-active-session")
            return
        }
        val apdu = intArrayToBytes(args.apdu)
        try {
            val response = when (handle) {
                is SessionHandle.Nfc -> handle.isoDep.transceive(apdu)
                is SessionHandle.Usb -> handle.connection.transmit(apdu)
            }
            invoke.resolve(JSObject().apply {
                put("response", bytesToJSArray(response))
            })
        } catch (e: Exception) {
            Log.d(TAG, "transmitApdu: failed", e)
            when (handle) {
                is SessionHandle.Nfc -> {
                    try { handle.isoDep.close() } catch (_: Exception) {}
                    sessions.values.removeIf { it is SessionHandle.Nfc && it.isoDep === handle.isoDep }
                    if (activeIsoDep === handle.isoDep) activeIsoDep = null
                }
                is SessionHandle.Usb -> {
                    try { handle.connection.close() } catch (_: Exception) {}
                    sessions.values.removeIf { it is SessionHandle.Usb && it.connection === handle.connection }
                    if (activeUsb === handle.connection) activeUsb = null
                }
            }
            invoke.reject(e.message ?: "transmit failed")
        }
    }

    @Command
    fun endSession(invoke: Invoke) {
        val args = invoke.parseArgs(EndSessionArgs::class.java)
        // Remove the session id but DO NOT close the physical handle
        // — the next beginSession in this flow reuses it.
        sessions.remove(args.sessionId)
        Log.d(TAG, "endSession: sessionId=${args.sessionId} (handle kept alive)")
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
