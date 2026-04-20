// SPDX-License-Identifier: GPL-3.0-or-later
//
// Android USB CCID transport for the tumpa smartcard plugin.
//
// The OpenPGP card spec on YubiKey 5 / Nitrokey 3 / any other CCID-
// class reader exposes a standard USB Chip Card Interface Device
// interface (class 0x0B). This file encapsulates the CCID framing —
// IccPowerOn, XfrBlock, IccPowerOff — so TumpaCardPlugin can treat
// USB identically to NFC at the APDU level.
//
// ## Wire format (USB CCID 1.1 §5)
//
// Every command/response starts with a 10-byte header, followed by
// an optional `abData` payload (APDU bytes going out; card response
// bytes coming back).
//
// Offset | Field            | Notes
// -------|------------------|---------------------------------------------
//  0     | bMessageType     | 0x62 IccPowerOn, 0x6F XfrBlock, 0x63 Off
//  1..4  | dwLength (LE)    | length of `abData`
//  5     | bSlot            | always 0 (readers like YubiKey have 1 slot)
//  6     | bSeq             | monotonically increasing; response echoes
//  7     | param 1          | e.g. bPowerSelect, bBWI
//  8..9  | param 2          | e.g. wLevelParameter for T=1 chaining
// 10..   | abData           | APDU / ATR bytes
//
// Response messages (0x80 DataBlock, 0x81 SlotStatus) use the same
// header. `bStatus` lives at offset 7 in responses:
//
//   bits 0..1 — 00=success, 01=failed, 10=time-extension request
//   bit  6    — ICC present / active
//
// If we get a time-extension request (card is slow — e.g. RSA key
// generation), we keep reading from bulk IN; the reader will send
// another DataBlock when the card answers.
//
// ## Permission flow
//
// `UsbManager.requestPermission(device, pendingIntent)` shows the
// system dialog. The result arrives async via a broadcast with
// `EXTRA_PERMISSION_GRANTED`. We keep the `onReady` callback in a
// ConcurrentHashMap keyed by device name so multiple concurrent
// requests don't clobber each other (unlikely in practice — there's
// only one card flow at a time — but cheap insurance).

package `in`.kushaldas.tumpa.card

import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.hardware.usb.UsbConstants
import android.hardware.usb.UsbDevice
import android.hardware.usb.UsbDeviceConnection
import android.hardware.usb.UsbEndpoint
import android.hardware.usb.UsbInterface
import android.hardware.usb.UsbManager
import android.os.Build
import android.util.Log
import java.io.IOException
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger

private const val TAG = "TumpaCardUsb"

/// Android-broadcast action scoped to our plugin. Must match what we
/// hand to `UsbManager.requestPermission()` so our `BroadcastReceiver`
/// actually receives the grant result.
private const val ACTION_USB_PERMISSION = "in.kushaldas.tumpa.card.USB_PERMISSION"

/// USB smartcard interface class (CCID). See USB-IF class codes.
private const val USB_CLASS_CCID = 0x0B

/// CCID message types we use (host → reader).
private const val MSG_ICC_POWER_ON = 0x62
private const val MSG_ICC_POWER_OFF = 0x63
private const val MSG_XFR_BLOCK = 0x6F
private const val MSG_GET_SLOT_STATUS = 0x65

/// CCID response types (reader → host).
private const val RSP_DATA_BLOCK = 0x80
private const val RSP_SLOT_STATUS = 0x81

/// bStatus lower-2-bit codes.
private const val STATUS_SUCCESS = 0
private const val STATUS_FAILED = 1
private const val STATUS_TIME_EXT = 2

/// How long a single bulk transfer can wait (ms). Generous because
/// on-card crypto (RSA key gen, signing large digests) can take a few
/// seconds; CCID's time-extension mechanism means the reader sends us
/// intermediate responses, but each one still counts as a round trip.
private const val BULK_TIMEOUT_MS = 30000

/// USB bulk packet size ceiling for full-speed devices is 64, for
/// high-speed 512. Reading in 512-byte chunks covers both; Android
/// just returns whatever landed.
private const val BULK_CHUNK = 512

/// Encapsulates one open CCID session: UsbDeviceConnection, claimed
/// interface, bulk IN/OUT endpoints, monotonic sequence counter.
class UsbCcidConnection private constructor(
    private val connection: UsbDeviceConnection,
    private val iface: UsbInterface,
    private val bulkIn: UsbEndpoint,
    private val bulkOut: UsbEndpoint,
) {
    private val seq = AtomicInteger(0)

    /// Historical bytes / ATR returned by the initial IccPowerOn.
    /// Stored so `beginSession` can echo it back in the
    /// `BeginSessionResponse.atr` field.
    var atr: ByteArray = ByteArray(0)
        private set

    /// AID of the applet the caller most recently selected. We need
    /// this so `recyclePowerAfterReset` can re-SELECT the applet after
    /// power-cycling the card, matching the behavior libtumpa expects
    /// (TERMINATE DF + ACTIVATE FILE returning 9000 with the applet
    /// still available for further commands).
    private var selectedAid: ByteArray? = null

    /// True while this connection is usable. Cleared on `close()` or
    /// any transport-level failure.
    @Volatile
    var isOpen: Boolean = true
        private set

    /// Record the AID the caller just SELECTed so the recycle path can
    /// re-SELECT the same applet after a reset-triggered power cycle.
    fun rememberSelectedAid(aid: ByteArray) {
        selectedAid = aid
    }

    /// Last successful VERIFY APDU per PIN reference (P2 byte: 0x81 =
    /// PW1 user, 0x82 = reset code, 0x83 = PW3 admin). YubiKey's CCID
    /// firmware fires a card-reset signal when a PUT DATA imports a
    /// large key; the recycle path re-SELECTs the applet but PIN state
    /// is lost, so a follow-up PUT DATA answers 6982. Replaying the
    /// remembered VERIFYs transparently re-arms the state.
    private val lastVerify = java.util.concurrent.ConcurrentHashMap<Byte, ByteArray>()

    /// Send IccPowerOn, capture the ATR. Must be called once after
    /// claiming the interface.
    fun powerOn() {
        val resp = exchange(buildIccPowerOn())
        atr = resp
        Log.d(TAG, "powerOn: ATR=${resp.toHexString()}")
    }

    /// Send one APDU wrapped in an XfrBlock. Returns the raw card
    /// response (including SW1 SW2 at the end — same shape as
    /// IsoDep.transceive).
    fun transmit(apdu: ByteArray): ByteArray {
        Log.d(TAG, "transmit apdu=${apdu.toHexString()}")
        val resp = try {
            exchange(buildXfrBlock(apdu))
        } catch (e: CardResetDuringExchange) {
            // Card reported "ICC inactive / reset" mid-APDU — triggered
            // either by the ACTIVATE FILE that closes a factory-reset
            // sequence, or by a PUT DATA that imports a key (YubiKey's
            // CCID firmware signals a reset when swapping key slots).
            // Recycle the slot and replay remembered state, then
            // synthesize a 9000 for the triggering command since the
            // card completed the operation from its own perspective.
            Log.d(TAG, "transmit: card reset detected, recycling slot")
            this.atr = recyclePowerAfterReset()
            byteArrayOf(0x90.toByte(), 0x00)
        }
        rememberIfVerify(apdu, resp)
        Log.d(TAG, "transmit resp=${resp.toHexString()} (${resp.size} bytes)")
        return resp
    }

    /// If the APDU we just sent was a successful VERIFY (ISO 7816-4
    /// CLA=00, INS=20, SW=9000), cache it so the reset-recycle path
    /// can replay it and restore the card's authenticated state.
    private fun rememberIfVerify(apdu: ByteArray, resp: ByteArray) {
        if (apdu.size < 4) return
        if (apdu[0] != 0x00.toByte() || apdu[1] != 0x20.toByte()) return
        if (resp.size < 2) return
        val sw1 = resp[resp.size - 2].toInt() and 0xff
        val sw2 = resp[resp.size - 1].toInt() and 0xff
        if (sw1 != 0x90 || sw2 != 0x00) return
        lastVerify[apdu[3]] = apdu.copyOf()
    }

    /// Slot recycle used after a card-reset response. YubiKey needs
    /// IccPowerOff followed by IccPowerOn; the first IccPowerOn right
    /// after ACTIVATE FILE returns SlotStatus with iccStatus=1 (card
    /// present but inactive), presumably because the reader's slot
    /// state machine is still draining the previous session. PowerOff
    /// flushes that, then PowerOn lights the slot up again.
    ///
    /// After the slot is back up we also re-SELECT whichever applet
    /// the caller had active, so subsequent APDUs from libtumpa land
    /// on the right applet (otherwise the card's default application
    /// answers with 6D00 "INS not supported").
    private fun recyclePowerAfterReset(): ByteArray {
        // Best-effort PowerOff; don't care if it fails — the slot is
        // already in a weird state.
        try {
            val off = buildHeader(MSG_ICC_POWER_OFF, 0)
            bulkWrite(off)
            // Drain the PowerOff's SlotStatus response; ignore contents.
            try { bulkReadMessage() } catch (_: Exception) {}
        } catch (e: Exception) {
            Log.d(TAG, "recyclePowerAfterReset: PowerOff threw (ignored)", e)
        }

        var atrResp: ByteArray? = null
        var lastError: Exception? = null
        for (attempt in 1..5) {
            try {
                Thread.sleep(100L * attempt)
                atrResp = exchange(buildIccPowerOn())
                break
            } catch (e: Exception) {
                Log.d(TAG, "recyclePowerAfterReset: IccPowerOn attempt $attempt failed", e)
                lastError = e
            }
        }
        val atr = atrResp ?: throw IOException(
            "card did not come back online after reset: ${lastError?.message}",
            lastError,
        )

        // Re-SELECT the applet we had open before the reset so libtumpa's
        // next APDU doesn't hit the (empty) default application. We
        // ignore the SELECT response — any non-9000 will surface later
        // as a normal card error from the next real APDU.
        selectedAid?.let { aid ->
            try {
                val selectCmd = buildSelectApdu(aid)
                val selResp = exchange(buildXfrBlock(selectCmd))
                val sw1 = selResp[selResp.size - 2].toInt() and 0xff
                val sw2 = selResp[selResp.size - 1].toInt() and 0xff
                Log.d(TAG, "recyclePowerAfterReset: re-SELECT sw=${"%02X%02X".format(sw1, sw2)}")
            } catch (e: Exception) {
                Log.w(TAG, "recyclePowerAfterReset: re-SELECT failed", e)
            }
        }

        // Replay every VERIFY that had previously succeeded. Card
        // resets clear PIN-verified state, so without this a follow-up
        // PUT DATA returns 6982 ("security status not satisfied") even
        // though the caller already authenticated.
        for ((p2, verifyApdu) in lastVerify) {
            try {
                val r = exchange(buildXfrBlock(verifyApdu))
                val sw1 = r[r.size - 2].toInt() and 0xff
                val sw2 = r[r.size - 1].toInt() and 0xff
                Log.d(TAG, "recyclePowerAfterReset: replay VERIFY P2=%02X sw=%02X%02X".format(p2, sw1, sw2))
            } catch (e: Exception) {
                Log.w(TAG, "recyclePowerAfterReset: replay VERIFY P2=%02X failed".format(p2), e)
            }
        }
        return atr
    }

    private fun buildSelectApdu(aid: ByteArray): ByteArray {
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

    private fun buildXfrBlock(apdu: ByteArray): ByteArray {
        val header = buildHeader(MSG_XFR_BLOCK, apdu.size, bBWI = 0x00)
        val cmd = ByteArray(header.size + apdu.size)
        System.arraycopy(header, 0, cmd, 0, header.size)
        System.arraycopy(apdu, 0, cmd, header.size, apdu.size)
        return cmd
    }

    private fun buildIccPowerOn(): ByteArray =
        buildHeader(MSG_ICC_POWER_ON, 0, bPowerSelect = 0x00)

    /// Thrown when the reader reports the card was reset / removed in
    /// the middle of a command. Signals the caller to re-power.
    private class CardResetDuringExchange : IOException("card reset during CCID exchange")

    /// Send IccPowerOff and release the interface. Idempotent.
    fun close() {
        if (!isOpen) return
        isOpen = false
        try {
            val cmd = buildHeader(MSG_ICC_POWER_OFF, 0)
            bulkWrite(cmd)
            // We don't bother reading the SlotStatus response here —
            // best-effort tear-down.
        } catch (e: Exception) {
            Log.d(TAG, "close: IccPowerOff failed (ignored)", e)
        }
        try { connection.releaseInterface(iface) } catch (_: Exception) {}
        try { connection.close() } catch (_: Exception) {}
    }

    // -- internals ----------------------------------------------------

    private fun buildHeader(
        messageType: Int,
        dataLen: Int,
        bPowerSelect: Int = 0,
        bBWI: Int = 0,
    ): ByteArray {
        val s = seq.getAndIncrement() and 0xff
        // 10-byte CCID header. Fields beyond the common first 7 vary
        // by message type; we only use IccPowerOn's bPowerSelect and
        // XfrBlock's bBWI. wLevelParameter stays 0 (short APDU, no
        // T=1 chaining at the host — YubiKey handles that internally).
        val p1: Int
        val p2a: Int
        val p2b: Int
        when (messageType) {
            MSG_ICC_POWER_ON -> { p1 = bPowerSelect and 0xff; p2a = 0; p2b = 0 }
            MSG_XFR_BLOCK -> { p1 = bBWI and 0xff; p2a = 0; p2b = 0 }
            else -> { p1 = 0; p2a = 0; p2b = 0 }
        }
        return byteArrayOf(
            (messageType and 0xff).toByte(),
            (dataLen and 0xff).toByte(),
            ((dataLen shr 8) and 0xff).toByte(),
            ((dataLen shr 16) and 0xff).toByte(),
            ((dataLen shr 24) and 0xff).toByte(),
            0,                // bSlot
            s.toByte(),       // bSeq
            p1.toByte(),
            p2a.toByte(),
            p2b.toByte(),
        )
    }

    /// Write the command, then read until we have a DataBlock whose
    /// bStatus is not "time extension". Returns `abData`.
    private fun exchange(cmd: ByteArray): ByteArray {
        bulkWrite(cmd)
        while (true) {
            val resp = bulkReadMessage()
            val type = resp[0].toInt() and 0xff
            val status = resp[7].toInt() and 0xff
            val cmdStatus = status and 0x03
            val iccStatus = (status shr 6) and 0x03
            Log.d(TAG, "ccid resp: type=0x%02X status=0x%02X cmdStatus=%d iccStatus=%d".format(type, status, cmdStatus, iccStatus))
            when (cmdStatus) {
                STATUS_SUCCESS -> {
                    if (type != RSP_DATA_BLOCK && type != RSP_SLOT_STATUS) {
                        throw IOException("unexpected CCID response type 0x%02X".format(type))
                    }
                    // bmICCStatus: 0=present+active, 1=present+inactive,
                    // 2=not present. YubiKey signals an in-flight reset
                    // (TERMINATE DF + ACTIVATE FILE) with iccStatus=2 and
                    // dwLength=0 — raise a typed exception so the
                    // transmit() wrapper can re-power the card and
                    // synthesize the 9000 that the command semantically
                    // completed.
                    if (iccStatus != 0) {
                        throw CardResetDuringExchange()
                    }
                    if (type == RSP_SLOT_STATUS) {
                        throw IOException("CCID returned SlotStatus (no data) when APDU response expected; status=0x%02X".format(status))
                    }
                    val len = (resp[1].toInt() and 0xff) or
                        ((resp[2].toInt() and 0xff) shl 8) or
                        ((resp[3].toInt() and 0xff) shl 16) or
                        ((resp[4].toInt() and 0xff) shl 24)
                    if (10 + len > resp.size) {
                        throw IOException("CCID short read: header says $len payload bytes, got ${resp.size - 10}")
                    }
                    if (len == 0) {
                        throw IOException("CCID DataBlock with dwLength=0 (card returned no data)")
                    }
                    return resp.copyOfRange(10, 10 + len)
                }
                STATUS_TIME_EXT -> {
                    // Card is busy — reader will send another message
                    // when it's ready. Loop and re-read.
                    Log.d(TAG, "exchange: time-extension request, re-reading")
                    continue
                }
                else -> {
                    val err = resp[8].toInt() and 0xff
                    throw IOException("CCID command failed: status=0x%02X error=0x%02X".format(status, err))
                }
            }
        }
    }

    private fun bulkWrite(data: ByteArray) {
        var offset = 0
        while (offset < data.size) {
            val n = connection.bulkTransfer(
                bulkOut,
                data,
                offset,
                data.size - offset,
                BULK_TIMEOUT_MS,
            )
            if (n < 0) {
                isOpen = false
                throw IOException("bulkTransfer OUT failed (returned $n)")
            }
            offset += n
        }
    }

    /// Read one complete CCID message by concatenating bulk IN packets
    /// until we have the full `10 + dwLength` bytes.
    private fun bulkReadMessage(): ByteArray {
        val first = ByteArray(BULK_CHUNK)
        val firstN = connection.bulkTransfer(bulkIn, first, first.size, BULK_TIMEOUT_MS)
        if (firstN < 10) {
            isOpen = false
            throw IOException("bulk IN returned $firstN bytes, expected ≥10 for CCID header")
        }
        val dwLength = (first[1].toInt() and 0xff) or
            ((first[2].toInt() and 0xff) shl 8) or
            ((first[3].toInt() and 0xff) shl 16) or
            ((first[4].toInt() and 0xff) shl 24)
        val total = 10 + dwLength
        if (firstN >= total) {
            return first.copyOfRange(0, total)
        }
        val out = ByteArray(total)
        System.arraycopy(first, 0, out, 0, firstN)
        var got = firstN
        while (got < total) {
            val chunk = ByteArray(BULK_CHUNK)
            val n = connection.bulkTransfer(bulkIn, chunk, chunk.size, BULK_TIMEOUT_MS)
            if (n <= 0) {
                isOpen = false
                throw IOException("bulk IN continuation returned $n after $got/$total bytes")
            }
            val copy = minOf(n, total - got)
            System.arraycopy(chunk, 0, out, got, copy)
            got += copy
        }
        return out
    }

    private fun ByteArray.toHexString(): String =
        joinToString("") { "%02X".format(it.toInt() and 0xff) }

    companion object {
        /// Enumerate USB devices and return the first one exposing a
        /// CCID interface, or null. Vendor/product IDs are
        /// deliberately ignored — we accept any standard CCID reader.
        fun findAttachedCcidDevice(manager: UsbManager): UsbDevice? {
            for (device in manager.deviceList.values) {
                if (findCcidInterface(device) != null) return device
            }
            return null
        }

        fun findCcidInterface(device: UsbDevice): UsbInterface? {
            for (i in 0 until device.interfaceCount) {
                val iface = device.getInterface(i)
                if (iface.interfaceClass == USB_CLASS_CCID) return iface
            }
            return null
        }

        /// Open the CCID interface on `device`. Assumes permission has
        /// already been granted (`manager.hasPermission(device)`).
        fun open(manager: UsbManager, device: UsbDevice): UsbCcidConnection {
            val iface = findCcidInterface(device)
                ?: throw IOException("device has no CCID interface")
            val conn = manager.openDevice(device)
                ?: throw IOException("failed to open USB device")
            if (!conn.claimInterface(iface, true)) {
                conn.close()
                throw IOException("failed to claim CCID interface")
            }
            var bulkIn: UsbEndpoint? = null
            var bulkOut: UsbEndpoint? = null
            for (i in 0 until iface.endpointCount) {
                val ep = iface.getEndpoint(i)
                if (ep.type == UsbConstants.USB_ENDPOINT_XFER_BULK) {
                    if (ep.direction == UsbConstants.USB_DIR_IN) bulkIn = ep
                    else bulkOut = ep
                }
            }
            if (bulkIn == null || bulkOut == null) {
                conn.releaseInterface(iface)
                conn.close()
                throw IOException("CCID interface missing bulk IN/OUT endpoints")
            }
            val ccid = UsbCcidConnection(conn, iface, bulkIn, bulkOut)
            ccid.powerOn()
            return ccid
        }
    }
}

/// Helper that wraps the async `UsbManager.requestPermission` flow in
/// a single-shot callback. Registers a broadcast receiver scoped to
/// `ACTION_USB_PERMISSION`, fires the permission request, and invokes
/// `onResult(true/false)` when the system dialog is dismissed.
///
/// If permission is already granted (`manager.hasPermission(device)`)
/// this calls `onResult(true)` synchronously without showing a dialog.
class UsbPermissionRequester(private val context: Context) {

    /// Pending callbacks keyed by device name. Android devices have a
    /// unique `/dev/bus/usb/…` path per device so this is a stable
    /// key while the device is attached.
    private val pending = ConcurrentHashMap<String, (Boolean) -> Unit>()

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(ctx: Context, intent: Intent) {
            Log.d(TAG, "permission receiver fired: action=${intent.action}")
            if (intent.action != ACTION_USB_PERMISSION) return
            val device: UsbDevice? = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                intent.getParcelableExtra(UsbManager.EXTRA_DEVICE, UsbDevice::class.java)
            } else {
                @Suppress("DEPRECATION")
                intent.getParcelableExtra(UsbManager.EXTRA_DEVICE)
            }
            val granted = intent.getBooleanExtra(UsbManager.EXTRA_PERMISSION_GRANTED, false)
            val name = device?.deviceName
            Log.d(TAG, "permission result: device=$name granted=$granted")
            if (name == null) {
                // Malformed broadcast — the system didn't attach EXTRA_DEVICE.
                // Don't leave any request hanging: clear every pending
                // callback with a denial so the caller's UI unblocks.
                val drained = pending.keys.toList()
                for (k in drained) pending.remove(k)?.invoke(false)
                Log.w(TAG, "permission result: no EXTRA_DEVICE, drained ${drained.size} pending")
                return
            }
            val cb = pending.remove(name) ?: run {
                Log.w(TAG, "permission result: no pending callback for $name")
                return
            }
            cb(granted)
        }
    }

    private var registered = false

    @Synchronized
    fun request(manager: UsbManager, device: UsbDevice, onResult: (Boolean) -> Unit) {
        if (manager.hasPermission(device)) {
            Log.d(TAG, "permission already granted for ${device.deviceName}")
            onResult(true)
            return
        }
        if (!registered) {
            val filter = IntentFilter(ACTION_USB_PERMISSION)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                // The permission grant broadcast is fired via PendingIntent
                // by system_server (UID 1000), so our receiver sees it as
                // coming from "another app" on API 33+. RECEIVER_EXPORTED
                // is required or the broadcast is silently dropped.
                context.registerReceiver(receiver, filter, Context.RECEIVER_EXPORTED)
            } else {
                @Suppress("UnspecifiedRegisterReceiverFlag")
                context.registerReceiver(receiver, filter)
            }
            registered = true
            Log.d(TAG, "registered USB permission receiver")
        }
        pending[device.deviceName] = onResult
        // FLAG_MUTABLE is required: UsbManager fills in EXTRA_DEVICE and
        // EXTRA_PERMISSION_GRANTED as extras on the broadcast via
        // PendingIntent.send(Context, int, Intent). With FLAG_IMMUTABLE
        // those extras are silently dropped and the receiver gets
        // device=null, granted=false. FLAG_MUTABLE only exists on
        // API 31+; on older versions PendingIntents are mutable by
        // default, so no flag is needed.
        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        } else {
            PendingIntent.FLAG_UPDATE_CURRENT
        }
        val intent = PendingIntent.getBroadcast(
            context, 0, Intent(ACTION_USB_PERMISSION).setPackage(context.packageName), flags
        )
        Log.d(TAG, "calling UsbManager.requestPermission for ${device.deviceName}")
        manager.requestPermission(device, intent)
    }
}
