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
