// SPDX-License-Identifier: GPL-3.0-or-later
//
// Tauri mobile plugin — OpenPGP smartcard bridge.
//
// iOS transports:
//
// - **NFC** via CoreNFC (`NFCTagReaderSession` + `NFCISO7816Tag`) —
//   implemented. Works with any ISO 7816-4 contactless smartcard that
//   answers the OpenPGP applet AID (YubiKey 5 NFC, Nitrokey 3 NFC,
//   Gnuk, etc.). Requires the `com.apple.developer.nfc.readersession
//   .iso7816.select-identifiers` entitlement on the host app, listing
//   `D2760001240103040000000000000000`, plus `NFCReaderUsageDescription`
//   in Info.plist.
//
// - **USB-C** via `TKSmartCard` (CryptoTokenKit) — **not implemented**.
//   The `com.apple.smartcard` entitlement is not granted to generic
//   third-party App Store apps; it's reserved for MDM / enterprise
//   deployments. `beginSession(transport: "usb")` rejects with a
//   clear message. YubiKey's iOS app uses the MFi ExternalAccessory
//   route (`com.yubico.ylp`), but that's vendor-specific and doesn't
//   cover Nitrokey / generic CCID readers, so we skip it.
//
// Keyring (M6b): implemented with iOS Keychain + LAContext — see
// below. Independent of the APDU transport.

import CoreNFC
import Foundation
import LocalAuthentication
import Security
import SwiftRs
import Tauri
import UIKit

// -- APDU-side argument / response types --------------------------

struct BeginSessionArgs: Decodable {
  let transport: String          // "nfc" | "usb"
  let appletAid: [UInt8]
}

struct TransmitApduArgs: Decodable {
  let sessionId: String
  let apdu: [UInt8]
}

struct EndSessionArgs: Decodable {
  let sessionId: String
}

struct BeginSessionResponse: Encodable {
  let sessionId: String
  let atr: [UInt8]?
}

struct TransmitApduResponse: Encodable {
  let response: [UInt8]
}

// -- Keyring argument types (generic secrets) ---------------------

struct SaveSecretArgs: Decodable {
  let key: String
  let secret: [UInt8]
}

struct ReadSecretArgs: Decodable {
  let key: String
  let reason: String
}

struct ReadSecretResponse: Encodable {
  let secret: [UInt8]
}

struct ClearSecretArgs: Decodable {
  let key: String
}

// -- Keychain helpers ---------------------------------------------

/// Shared Keychain service name. Everything the plugin writes lives
/// under this service, so `clear_all_secrets` can wipe the whole
/// namespace in one `SecItemDelete` call.
private let KEYCHAIN_SERVICE = "in.kushaldas.tumpa.secret"

/// Build the access-control flags used on every secret entry.
/// Biometric (current enrollment) with devicePasscode fallback — if
/// Face ID is locked out (5 failed attempts) the user can still get
/// in with their passcode, which matters because the alternative is
/// permanent data loss on the device.
private func secretAccessControl() throws -> SecAccessControl {
  var error: Unmanaged<CFError>?
  guard
    let ac = SecAccessControlCreateWithFlags(
      nil,
      kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly,
      [.biometryCurrentSet, .or, .devicePasscode],
      &error)
  else {
    let cf = error?.takeRetainedValue()
    let msg = cf.map { CFErrorCopyDescription($0) as String? ?? "unknown" } ?? "nil"
    throw NSError(
      domain: "tumpa-card",
      code: 2,
      userInfo: [NSLocalizedDescriptionKey: "access control: \(msg)"])
  }
  return ac
}

private func keychainSave(key: String, secret: Data) throws {
  let access = try secretAccessControl()

  // Delete any prior entry — SecItemAdd fails with errSecDuplicateItem
  // if one already exists, and SecItemUpdate can't change access
  // control flags.
  let deleteQuery: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: KEYCHAIN_SERVICE,
    kSecAttrAccount as String: key,
  ]
  SecItemDelete(deleteQuery as CFDictionary)

  let addQuery: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: KEYCHAIN_SERVICE,
    kSecAttrAccount as String: key,
    kSecAttrAccessControl as String: access,
    kSecAttrSynchronizable as String: false,
    kSecValueData as String: secret,
  ]
  let status = SecItemAdd(addQuery as CFDictionary, nil)
  if status != errSecSuccess {
    throw NSError(
      domain: "tumpa-card",
      code: Int(status),
      userInfo: [NSLocalizedDescriptionKey: "SecItemAdd status \(status)"])
  }
}

private func keychainRead(key: String, reason: String) throws -> Data {
  let context = LAContext()
  context.localizedReason = reason
  context.localizedCancelTitle = "Cancel"

  let query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: KEYCHAIN_SERVICE,
    kSecAttrAccount as String: key,
    kSecReturnData as String: true,
    kSecUseAuthenticationContext as String: context,
    kSecUseOperationPrompt as String: reason,
  ]

  var result: CFTypeRef?
  let status = SecItemCopyMatching(query as CFDictionary, &result)

  switch status {
  case errSecSuccess:
    guard let data = result as? Data else {
      throw NSError(
        domain: "tumpa-card",
        code: 3,
        userInfo: [NSLocalizedDescriptionKey: "unexpected keychain value type"])
    }
    return data
  case errSecItemNotFound:
    throw NSError(
      domain: "tumpa-card",
      code: Int(errSecItemNotFound),
      userInfo: [NSLocalizedDescriptionKey: "no-secret-saved"])
  case errSecUserCanceled, errSecAuthFailed:
    throw NSError(
      domain: "tumpa-card",
      code: Int(status),
      userInfo: [NSLocalizedDescriptionKey: "cancelled"])
  default:
    throw NSError(
      domain: "tumpa-card",
      code: Int(status),
      userInfo: [NSLocalizedDescriptionKey: "SecItemCopyMatching status \(status)"])
  }
}

private func keychainClear(key: String) throws {
  let query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: KEYCHAIN_SERVICE,
    kSecAttrAccount as String: key,
  ]
  let status = SecItemDelete(query as CFDictionary)
  if status != errSecSuccess && status != errSecItemNotFound {
    throw NSError(
      domain: "tumpa-card",
      code: Int(status),
      userInfo: [NSLocalizedDescriptionKey: "SecItemDelete status \(status)"])
  }
}

private func keychainClearAll() throws {
  let query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: KEYCHAIN_SERVICE,
  ]
  let status = SecItemDelete(query as CFDictionary)
  if status != errSecSuccess && status != errSecItemNotFound {
    throw NSError(
      domain: "tumpa-card",
      code: Int(status),
      userInfo: [NSLocalizedDescriptionKey: "SecItemDelete status \(status)"])
  }
}

// -- NFC session ---------------------------------------------------

/// Physical Core NFC session. Multiple logical `sessionId`s (issued by
/// the plugin one-per-`beginSession`) can share the same physical
/// session, because libtumpa's higher-level flows (e.g. `upload`) call
/// several wecanencrypt ops back-to-back, each of which creates a
/// fresh `MobileCardBackend` that does `begin_session`→APDUs→
/// `end_session`. If we tore down the `NFCTagReaderSession` on every
/// `end_session`, the user would be forced to tap their card 3–5
/// times per upload — and iOS refuses to reopen another session
/// immediately anyway (it terminates the new one with
/// "Session invalidated unexpectedly").
///
/// So the plugin holds the `NFCTagReaderSession` + `NFCISO7816Tag`
/// warm. On `end_session` we just unlink the logical id; on the next
/// `begin_session` we re-issue SELECT on the already-connected tag
/// (via `rebind`) and resolve immediately — no new sheet. If no
/// follow-up `begin_session` arrives within the grace period (2s),
/// the plugin fires `end()` and the sheet dismisses naturally.
///
/// Thread model:
/// - `NFCTagReaderSession` callbacks are delivered on the
///   `DispatchQueue.global()` queue we hand it.
/// - `transmit(apdu:invoke:)` is called from Tauri's plugin worker;
///   we forward to `tag.sendCommand(apdu:completionHandler:)` which
///   resumes on the same background queue.
/// - Access to `tag`, `pendingBegin`, etc. is serialized on
///   `sessionQueue`.
@available(iOS 13.0, *)
private class NFCCardSession: NSObject, NFCTagReaderSessionDelegate {

  // IUO so we can set it post-super.init — NFCTagReaderSession's
  // initializer takes the delegate by strong reference and its
  // `delegate` property is get-only, so we must pass `self` at
  // construction, which in turn requires super.init() to have run.
  private var nfcSession: NFCTagReaderSession!
  private var tag: NFCISO7816Tag?
  private let aid: Data
  private let sessionQueue = DispatchQueue(label: "in.kushaldas.tumpa.card.nfc")

  /// The Invoke we still owe a response to for `begin_session`. Set at
  /// construction and on each `rebind`, cleared once we resolve/reject.
  private var pendingBegin: Invoke?

  /// The external `sessionId` we will hand back when the current
  /// `pendingBegin` resolves. Rotated by `rebind(newInvoke:sessionId:)`.
  private var pendingSessionId: String

  /// Weak back-pointer so the delegate callback that fires on
  /// `didInvalidateWithError` can clear the plugin's warm reference.
  weak var plugin: TumpaCardPlugin?

  init?(plugin: TumpaCardPlugin, aid: Data, beginInvoke: Invoke, sessionId: String) {
    self.aid = aid
    self.pendingBegin = beginInvoke
    self.pendingSessionId = sessionId
    self.plugin = plugin
    super.init()
    guard let session = NFCTagReaderSession(
      pollingOption: [.iso14443],
      delegate: self,
      queue: DispatchQueue.global(qos: .userInitiated))
    else { return nil }
    self.nfcSession = session
    self.nfcSession.alertMessage = "Hold your OpenPGP card near the top of the phone"
  }

  func start() {
    nfcSession.begin()
  }

  /// True if the Core NFC session still has a connected tag. Callers
  /// hit this to decide whether to reuse the warm session on the next
  /// `begin_session`.
  var hasLiveTag: Bool {
    return sessionQueue.sync { self.tag != nil }
  }

  /// Warm-path entry. The plugin keeps the NFC session open between
  /// wecanencrypt ops; each new `begin_session` re-SELECTs the applet
  /// on the already-connected tag and resolves synchronously.
  func rebind(newInvoke: Invoke, sessionId: String) {
    let tagRef: NFCISO7816Tag? = sessionQueue.sync {
      self.pendingBegin = newInvoke
      self.pendingSessionId = sessionId
      return self.tag
    }
    guard let tagRef = tagRef else {
      rejectBegin("warm-tag-lost")
      return
    }
    let selectApdu = NFCISO7816APDU(
      instructionClass: 0x00,
      instructionCode: 0xA4,
      p1Parameter: 0x04,
      p2Parameter: 0x00,
      data: self.aid,
      expectedResponseLength: 256)
    tagRef.sendCommand(apdu: selectApdu) { [weak self] _, sw1, sw2, apduError in
      guard let self = self else { return }
      if let apduError = apduError {
        self.rejectBegin(apduError.localizedDescription)
        return
      }
      if sw1 != 0x90 || sw2 != 0x00 {
        let sw = String(format: "%02X%02X", sw1, sw2)
        self.rejectBegin("SELECT failed \(sw)")
        return
      }
      self.resolveBegin()
    }
  }

  // MARK: - NFCTagReaderSessionDelegate

  func tagReaderSessionDidBecomeActive(_ session: NFCTagReaderSession) {
    // No-op — the OS sheet is up.
  }

  func tagReaderSession(_ session: NFCTagReaderSession, didDetect tags: [NFCTag]) {
    guard let first = tags.first, case let .iso7816(iso7816Tag) = first else {
      session.invalidate(errorMessage: "Not an ISO 7816 smartcard")
      return
    }
    session.connect(to: .iso7816(iso7816Tag)) { [weak self] connectError in
      guard let self = self else { return }
      if let connectError = connectError {
        session.invalidate(errorMessage: connectError.localizedDescription)
        self.rejectBegin(connectError.localizedDescription)
        return
      }

      // SELECT the OpenPGP applet. If the card doesn't have one, this
      // returns a non-0x9000 SW and we fail the begin call.
      let selectApdu = NFCISO7816APDU(
        instructionClass: 0x00,
        instructionCode: 0xA4,
        p1Parameter: 0x04,
        p2Parameter: 0x00,
        data: self.aid,
        expectedResponseLength: 256)

      iso7816Tag.sendCommand(apdu: selectApdu) { [weak self] _, sw1, sw2, apduError in
        guard let self = self else { return }
        if let apduError = apduError {
          session.invalidate(errorMessage: apduError.localizedDescription)
          self.rejectBegin(apduError.localizedDescription)
          return
        }
        if sw1 != 0x90 || sw2 != 0x00 {
          let sw = String(format: "%02X%02X", sw1, sw2)
          session.invalidate(errorMessage: "Card rejected OpenPGP SELECT (\(sw))")
          self.rejectBegin("SELECT failed \(sw)")
          return
        }

        // Stash the tag and update the sheet message; don't invalidate
        // the session — we want to keep it open for the follow-up
        // transmit APDUs.
        self.sessionQueue.sync {
          self.tag = iso7816Tag
        }
        session.alertMessage = "Connected — keep holding…"
        self.resolveBegin()
      }
    }
  }

  func tagReaderSession(_ session: NFCTagReaderSession, didInvalidateWithError error: Error) {
    let msg: String
    if let nfcError = error as? NFCReaderError,
       nfcError.code == .readerSessionInvalidationErrorUserCanceled
    {
      msg = "cancelled"
    } else {
      msg = error.localizedDescription
    }
    self.rejectBegin(msg)
    self.sessionQueue.sync {
      self.tag = nil
    }
    self.plugin?.nfcSessionInvalidated(session: self)
  }

  /// Resolve the `begin_session` invoke we kept pending since
  /// construction, if any.  No-op if it was already completed.
  private func resolveBegin() {
    let (invoke, sid): (Invoke?, String) = sessionQueue.sync {
      let i = self.pendingBegin
      self.pendingBegin = nil
      return (i, self.pendingSessionId)
    }
    invoke?.resolve(BeginSessionResponse(sessionId: sid, atr: nil))
  }

  /// Reject the pending `begin_session` invoke with `message`. No-op
  /// if it was already completed (e.g. invalidation after a
  /// successful SELECT is normal session end, not an error to
  /// propagate).
  private func rejectBegin(_ message: String) {
    let invoke: Invoke? = sessionQueue.sync {
      let i = self.pendingBegin
      self.pendingBegin = nil
      return i
    }
    invoke?.reject(message)
  }

  // MARK: - APDU transmit

  func transmit(apdu: Data, invoke: Invoke) {
    let tagRef: NFCISO7816Tag? = sessionQueue.sync { self.tag }
    guard let tagRef = tagRef else {
      invoke.reject("no-active-session")
      return
    }
    guard let parsed = NFCISO7816APDU(data: apdu) else {
      invoke.reject("invalid APDU")
      return
    }
    tagRef.sendCommand(apdu: parsed) { response, sw1, sw2, error in
      if let error = error {
        invoke.reject(error.localizedDescription)
        return
      }
      var combined = Data(response)
      combined.append(sw1)
      combined.append(sw2)
      invoke.resolve(TransmitApduResponse(response: Array(combined)))
    }
  }

  func end() {
    nfcSession.invalidate()
  }
}

// -- Plugin entry point -------------------------------------------

class TumpaCardPlugin: Plugin {

  /// Logical session-id → physical NFC session. Many keys can point at
  /// the same `NFCCardSession`; see the class comment on
  /// `NFCCardSession` for why we keep one physical session warm across
  /// multiple Rust-side begin/end cycles.
  private var sessions: [String: NFCCardSession] = [:]

  /// The currently-live Core NFC session, or nil if none. Any cold
  /// `beginSession` creates a fresh one; any warm `beginSession` reuses
  /// this via `rebind`.
  private var warmSession: NFCCardSession?

  /// Scheduled dismissal of `warmSession` after the last logical
  /// session ended. Cancelled by the next `beginSession` if it arrives
  /// inside the grace window. 2s is plenty: libtumpa's sequential ops
  /// are back-to-back Rust calls (microseconds apart); real user-visible
  /// gap only happens at the end of a high-level flow (upload done, card
  /// details fetched, etc.) and we want the sheet to dismiss then.
  private var dismissTimer: DispatchWorkItem?
  private let sessionsQueue = DispatchQueue(label: "in.kushaldas.tumpa.card.sessions")

  // MARK: - APDU bridge

  @objc public func beginSession(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(BeginSessionArgs.self)
    switch args.transport {
    case "nfc", "auto":
      // "auto" on iOS always resolves to NFC — the `com.apple.smartcard`
      // entitlement needed for `TKSmartCard` over USB-C isn't granted
      // to generic App Store apps, so there's no USB path to pick.
      beginNFCSession(aid: Data(args.appletAid), invoke: invoke)
    case "usb":
      invoke.reject(
        "USB-C smartcard on iOS requires the com.apple.smartcard entitlement, "
          + "which Apple does not grant to generic App Store apps. Use NFC instead.")
    default:
      invoke.reject("unknown transport: \(args.transport)")
    }
  }

  private func beginNFCSession(aid: Data, invoke: Invoke) {
    guard NFCTagReaderSession.readingAvailable else {
      invoke.reject("NFC is not available on this device")
      return
    }

    let newSessionId = UUID().uuidString

    // Warm path: a live physical session is still open from a prior
    // wecanencrypt op. Cancel the scheduled dismissal, re-SELECT the
    // applet on the existing tag, and resolve without prompting the
    // user for another tap.
    let warm: NFCCardSession? = sessionsQueue.sync {
      if let w = self.warmSession, w.hasLiveTag {
        self.dismissTimer?.cancel()
        self.dismissTimer = nil
        self.sessions[newSessionId] = w
        return w
      }
      return nil
    }
    if let warm = warm {
      warm.rebind(newInvoke: invoke, sessionId: newSessionId)
      return
    }

    // Cold path: create a new Core NFC session, show the system sheet.
    guard
      let session = NFCCardSession(
        plugin: self, aid: aid, beginInvoke: invoke, sessionId: newSessionId)
    else {
      invoke.reject("Could not create NFC session")
      return
    }
    sessionsQueue.sync {
      self.sessions[newSessionId] = session
      self.warmSession = session
    }
    session.start()
  }

  @objc public func transmitApdu(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(TransmitApduArgs.self)
    let session: NFCCardSession? = sessionsQueue.sync { self.sessions[args.sessionId] }
    guard let session = session else {
      invoke.reject("no-active-session")
      return
    }
    session.transmit(apdu: Data(args.apdu), invoke: invoke)
  }

  @objc public func endSession(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(EndSessionArgs.self)
    sessionsQueue.sync {
      self.sessions.removeValue(forKey: args.sessionId)

      // If no more logical sessions reference the warm physical session,
      // schedule a delayed `end()` so a follow-up `beginSession` within
      // the grace window can reuse it.
      guard let warm = self.warmSession else { return }
      let stillInUse = self.sessions.values.contains(where: { $0 === warm })
      if !stillInUse {
        self.dismissTimer?.cancel()
        let item = DispatchWorkItem { [weak self, weak warm] in
          guard let self = self else { return }
          self.sessionsQueue.sync {
            // Another begin may have reclaimed the warm session while
            // the timer was in flight; only dismiss if it's still idle.
            if let warm = warm,
               self.warmSession === warm,
               !self.sessions.values.contains(where: { $0 === warm })
            {
              warm.end()
              self.warmSession = nil
            }
            self.dismissTimer = nil
          }
        }
        self.dismissTimer = item
        DispatchQueue.global(qos: .userInitiated).asyncAfter(
          deadline: .now() + 2.0, execute: item)
      }
    }
    invoke.resolve()
  }

  /// Callback from `NFCCardSession.didInvalidateWithError` — the
  /// physical NFC session died (user cancelled, tag lost, iOS timeout).
  /// Clear every logical session that was mapped to it.
  fileprivate func nfcSessionInvalidated(session: NFCCardSession) {
    sessionsQueue.sync {
      self.sessions = self.sessions.filter { $0.value !== session }
      if self.warmSession === session {
        self.warmSession = nil
      }
      self.dismissTimer?.cancel()
      self.dismissTimer = nil
    }
  }

  // MARK: - Keyring-backed secret storage (M6b)

  @objc public func saveSecret(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(SaveSecretArgs.self)
    do {
      try keychainSave(key: args.key, secret: Data(args.secret))
      invoke.resolve()
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }

  @objc public func readSecret(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ReadSecretArgs.self)
    do {
      let data = try keychainRead(key: args.key, reason: args.reason)
      invoke.resolve(ReadSecretResponse(secret: Array(data)))
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }

  @objc public func clearSecret(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ClearSecretArgs.self)
    do {
      try keychainClear(key: args.key)
      invoke.resolve()
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }

  @objc public func clearAllSecrets(_ invoke: Invoke) throws {
    do {
      try keychainClearAll()
      invoke.resolve()
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }
}

@_cdecl("init_plugin_tumpa_card")
func initPlugin() -> Plugin {
  return TumpaCardPlugin()
}
