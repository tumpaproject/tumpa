# ADR-0002: Cold-Start Performance Strategy

## Status

Accepted

## Context

With a realistic keystore (~137 keys), the Tumpa window took roughly a
second to show anything useful on cold start, and the user saw either
a solid black rectangle (WebKitGTK default) or a plain white window
during that gap. Instrumentation showed the time was split roughly:

- **Rust `list_keys` command** — ~700 ms on a 137-key store. Every
  invocation re-parsed every OpenPGP blob through rpgp to rebuild
  `KeyInfo`, plus one `SELECT` against `card_keys` per key (a
  classic N+1).
- **Two `refreshKeys` calls per cold start** — `SidebarLayout.onMounted`
  and the route view (`KeyListView`) both fired their own refresh,
  back-to-back and sequentially.
- **~700 ms of WebKitGTK paint cost** — the desktop list rendered
  every `<KeyItem>` component for every key at once; WebKitGTK is
  substantially slower than Blink/Gecko at painting dense lists.
- **~400 ms gap with nothing on screen** — WebKit's default window
  background (black/white depending on compositor) showed through
  until Vue's first paint committed.

The list-view caller only needs fingerprint, primary UID name/email,
creation / expiry / revocation dates, an algorithm label, and the
set of linked cards. wecanencrypt already normalised most of that
into the `keys` + `user_ids` + `subkeys` + `card_keys` tables at
import time — the full rpgp parse per row was recovering data we had
already committed to SQL.

## Decision

Four layers of fix, each addressing a different part of the cold-start
budget. All four landed together so the wins compound.

### 1. Read the cache instead of re-parsing (wecanencrypt + libtumpa)

Schema v4 (wecanencrypt 0.12.1) extends the `keys` table with
`is_revoked`, `revocation_time`, `expiration_time`, `cert_created_at`
and adds `algorithm` + `bit_length` columns to `subkeys`. Existing
rows are backfilled by parsing each blob once at migration time; all
future `import_key` paths populate the columns inline. New APIs
expose the cached view:

- `KeyStore::list_keys_summary()` → one SELECT per table, no blob
  parse.
- `KeyStore::get_key_summary(fingerprint)` → single-row variant.
- `KeyStore::list_all_card_keys()` → bulk getter so callers can
  build a fingerprint → cards map with one query instead of N.

libtumpa 0.1.3 adds `card::link::card_idents_map(&KeyStore)` that
uses the new bulk getter to produce a deduped map in one round trip.

Tumpa registers a new `list_keys_summary` Tauri command that zips
the summary rows with the card map and returns a `KeyListRow` slim
enough for the list view. `store.refreshKeys` now invokes this
command instead of the parse-heavy `list_keys`. Detail screens still
call `get_key_details` / `get_key_info` and pay the full parse cost
on demand, which is the right trade-off — users only open one detail
view at a time.

### 2. Paint first, fetch in the background

`SidebarLayout.onMounted` calls `store.refreshKeys()` without
`await`ing it, then redirects to `/keys` if the store turned out to
have keys *and* we're still on `/`. Vue's first render commits the
sidebar + current route in the very first frame; the Rust IPC
resolves in the background and reconciles the view when it returns.
Card detection (PCSC) stays deferred with `requestIdleCallback` so
the first paint doesn't block on it.

`StartView` renders its empty-state UI unconditionally — no `v-if`
ready-gate. On a cold start with keys the redirect typically fires
before WebKit commits a second frame, so the brief "No keys added
yet" flash is usually imperceptible; if a user does catch it, the
trade-off is worth avoiding the ~400 ms white content area we saw
with the gate in place.

An in-flight dedup in `appStore.refreshKeys` collapses overlapping
callers into a single IPC; and `KeyListView` / `KeyListMobile`
`onMounted` skip their own refresh when `store.keysLoaded` is already
true, so the `/` → `/keys` redirect chain no longer causes a second
identical invocation.

### 3. Virtualise the list

Both the desktop `KeyListView` (`DynamicScroller`) and the mobile
`KeyListMobile` (`RecycleScroller`) use `vue-virtual-scroller`. Only
the ~5-10 rows that actually fit in the viewport are in the DOM at
any moment. Scaling the keystore no longer scales WebKitGTK's
first-paint cost.

Key-list rows are also wrapped with `markRaw` in the store so Vue /
Pinia does not wrap every row field in a reactive proxy — the rows
are read-only snapshots from Rust and nothing in the UI mutates them
in place.

### 4. Branded boot splash over a matching window background

A full-viewport `#tumpa-boot` splash is inlined into `index.html` —
the Tumpa wordmark SVG centered on the sidebar-purple background,
with a gentle opacity pulse. Because it lives outside `#app`, it is
not replaced by Vue's mount; `main.js` instead holds it for a
minimum of 400 ms, then fades it out over 220 ms and removes the
node. Tauri's `windows[0].backgroundColor` is set to the same
purple so the pre-paint WebKit frame matches the splash, removing
the black/white flash that was visible on fast release builds.

### Debug-only instrumentation

`[tumpa/perf]` timing logs live in both the Rust command
(`#[cfg(debug_assertions)]`) and the JS bootstrap / store
(`import.meta.env.DEV`). Production bundles contain neither the
`eprintln!`s nor the `console.log`s; Vite statically removes the
dead branch.

## Consequences

- Cold-start first-useful-paint for a 137-key store drops from
  roughly 1 s to the ~400 ms splash floor plus fade (intentional,
  branded). Actual Vue render is visible immediately on splash
  removal.
- Adding new keys to the store no longer makes cold start slower
  in a way proportional to count — virtualization caps the DOM
  budget, and the summary API reads in constant SQL query count.
- Pre-paint window flashes (white/black) are eliminated on
  platforms that respect Tauri's `backgroundColor` (all desktop
  targets) and masked by the splash on platforms that don't.
- Mobile follows the same virtualization + `keysLoaded` dedup; the
  splash only needs to cover desktop because the mobile cold path
  already had less time-to-first-paint and its own `ready` gate in
  `StartMobile`.
- The list view now depends on summary columns populated by
  wecanencrypt schema v4. Any wecanencrypt downgrade to pre-0.12.1
  would reintroduce the parse-heavy path; the `libtumpa` 0.1.3
  dependency in `src-tauri/Cargo.toml` guards against that.
- Fields not cached by the summary API (per-UID revocation, UID
  certifications, subkey bit-for-bit fingerprints beyond what
  `subkeys` stores) are only available on the detail screens,
  which call `get_key_details`. The list view surfaces
  whole-cert revocation only.
- Virtualization means `Ctrl+F` / browser find-in-page no longer
  matches off-screen keys. The in-app search box in the list
  header is the substitute for that, and already filters the
  underlying array before it reaches the scroller.
- The 400 ms splash floor is a deliberate UX floor, not a
  technical minimum. On very fast release builds the real UI is
  ready well before 400 ms; the floor ensures users see a branded
  splash rather than a perceptual flash. Tune via `MIN_SPLASH_MS`
  in `src/main.js`.

## Trade-offs considered but not taken

- **Make the list-view payload a streaming IPC** (first N rows
  eagerly, rest on demand). Rejected — at summary-API speeds the
  whole list returns in a few ms; streaming would be strictly
  more complexity for no visible win.
- **Server-side rendering / pre-hydration of the Vue tree.** Not
  a fit for a Tauri desktop app; there is no server, and the
  boot splash achieves the same "something branded on the first
  frame" goal with zero build-system complexity.
- **Replace rpgp with a faster parser.** Dismissed — the whole
  point of §1 is that we don't need to parse on the hot path;
  parser speed becomes irrelevant for the list view. Detail-view
  parse time is already fast enough that users don't notice.
- **Fixed-height `RecycleScroller` on desktop.** Considered for
  the extra ~80-120 ms paint saving, but `KeyItem` height varies
  with UID count and clamping it would force UID truncation —
  an accessibility / information-density regression that isn't
  worth the millisecond savings on top of virtualization.
