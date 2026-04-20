/**
 * Returns true when the app is running inside a mobile webview
 * (Tauri Android / iOS) or in a narrow browser window.
 *
 * Called from both main.js (to prime the store flag) and router/index.js
 * (to choose mobile vs desktop route components). Must be cheap and
 * synchronous because both callers run during module evaluation.
 */
export function isMobilePlatform() {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent.toLowerCase()
  if (/android|iphone|ipad|ipod/.test(ua)) return true
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(max-width: 768px)').matches
  }
  return false
}

/**
 * Returns true when the app is running on iOS. Used to hide the
 * USB transport options — Apple doesn't grant third-party apps the
 * `com.apple.smartcard` entitlement needed for `TKSmartCard`, so
 * only NFC is actually wired up on iOS.
 */
export function isIosPlatform() {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent.toLowerCase()
  return /iphone|ipad|ipod/.test(ua)
}
