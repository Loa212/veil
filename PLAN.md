# Veil — Build Plan / Claude Code Spec

A macOS "soft lockscreen" that drops a fullscreen overlay the instant the app
loses focus, prompts for Touch ID or a custom PIN, and falls back to the real
macOS lock screen if auth fails or is dismissed. Built for the
"step-away-from-my-desk-without-colleagues-snooping" use case.

> Working name: **Veil**. Rename freely (`Blinds`, `Sentry`, `Vigil` were candidates).

---

## 0. Decisions locked in (do not re-litigate)

| Decision         | Choice                                                                                                                         |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Framework        | **Tauri 2.x** (Rust core + web frontend)                                                                                       |
| Min macOS        | **Sequoia 15+** only                                                                                                           |
| Auth             | **Touch ID** (LocalAuthentication) **+ custom PIN + recovery code** (Keychain). NO macOS-password validation.                  |
| Re-arm behavior  | Lock once on failed/dismissed auth, then **freeze** (overlay tears down, blur listener disarms). Manual "Resume" re-arms.      |
| Nuclear fallback | Trigger **native macOS lock screen** (Cmd+Ctrl+Q equivalent).                                                                  |
| Menubar          | **arm / disarm / status** only.                                                                                                |
| Settings         | Real settings **window**, opened from menubar.                                                                                 |
| Distribution     | **Homebrew cask** + signed/notarized **.dmg**. (Docker dropped — impossible for a Mac GUI app.)                                |
| CLI              | Minimal: `veil` launches the app / `veil arm` / `veil status`. Thin wrapper, no heavy runtime control.                         |
| MVP overlay      | Static image background + clock + date (native-lockscreen style). Hint text & animated bg = post-MVP, but architect modularly. |

---

## 1. Architecture overview

```
┌─────────────────────────────────────────────────────────┐
│  Tauri app (single process)                              │
│                                                          │
│  Rust core (src-tauri/)                                  │
│   ├─ tray: arm/disarm/status menu                        │
│   ├─ focus watcher: NSApplication didResignActive        │
│   ├─ overlay manager: 1 window per NSScreen              │
│   ├─ auth: LocalAuthentication (Touch ID) + PIN verify   │
│   ├─ keychain: PIN hash + recovery code (Security fwk)   │
│   ├─ lock trigger: native lock screen fallback           │
│   ├─ power: prevent sleep while armed (IOKit assertion)  │
│   └─ settings store: JSON in app support dir             │
│                                                          │
│  Frontend (src/) — your comfort zone (React + TS + Vite) │
│   ├─ Overlay view (bg image + clock + date)              │
│   ├─ Auth prompt (PIN entry, Touch ID trigger, recovery) │
│   ├─ Settings window (PIN edit, hint, bg picker, timeout)│
│   └─ First-run setup (set PIN, generate recovery code)   │
└─────────────────────────────────────────────────────────┘
```

The interesting/hard parts are all macOS-native and live in Rust via the
`objc2` / `cocoa` crates (or `objc2-local-authentication`,
`objc2-app-kit`, `objc2-foundation`). The frontend is plain web — that's
where you move fast.

---

## 2. The hard macOS-native pieces (research these FIRST in Claude Code)

These are the parts that will eat the time. Prototype each in isolation
before wiring the full app.

### 2.1 Focus-loss detection

- Hook `NSApplication`'s `applicationDidResignActive:` (or observe
  `NSApplicationDidResignActiveNotification`).
- This fires on Cmd+Tab away, clicking another app, Mission Control, etc.
- **Gotcha:** showing your own overlay window may itself steal/return focus.
  Distinguish "user left" from "we're presenting the overlay" with an
  internal `state` flag (`Idle | Armed | Presenting | Frozen`).
- Optional grace timeout (configurable, default 0s) before overlay arms,
  to avoid lockout loops.

### 2.2 Multi-display overlay

- Enumerate `NSScreen.screens` → spawn one borderless Tauri window per display,
  each sized/positioned to its screen frame.
- Window level: **`NSScreenSaverWindowLevel`** (or `kCGScreenSaverWindowLevel`)
  so it covers the menu bar and Dock. Tauri exposes window level via
  `objc2` on the underlying `NSWindow` (raw handle).
- Flags: `alwaysOnTop`, no shadow, ignore-mouse-events = false (we WANT to
  capture input to trigger auth), `collectionBehavior` =
  `canJoinAllSpaces | fullScreenAuxiliary | stationary` so it follows across
  spaces.
- Auth prompt renders on the **primary display only** (Touch ID is tied to it);
  other displays just show the background + clock.

### 2.3 "Any input wakes the auth prompt"

- Earlier idea: trigger auth on _any_ mouse move / keypress while overlay shown.
- Implement with a local event monitor (`NSEvent.addLocalMonitorForEvents`)
  for `keyDown | mouseMoved | leftMouseDown` while Presenting.
- MVP simplification: overlay is up → first interaction surfaces the PIN/Touch ID
  prompt. Don't over-engineer; a click-to-reveal is fine for v1.

### 2.4 Touch ID

- `objc2-local-authentication`: `LAContext`, `canEvaluatePolicy:`,
  `evaluatePolicy:localizedReason:reply:`.
- Policy: `.deviceOwnerAuthenticationWithBiometrics` (biometrics only) with
  PIN as the in-app fallback (NOT `.deviceOwnerAuthentication`, which would
  surface the _system_ password — we don't want that).

### 2.5 PIN + recovery storage

- Store **Argon2id hash** of PIN (not plaintext) — use the `argon2` crate.
- Store hash + a generated recovery code in the **macOS Keychain** via the
  Security framework (`security-framework` crate), item protected so it's
  bound to this app.
- Recovery code: generated once at setup, shown to user to save/print,
  also stored hashed. Entering it bypasses a forgotten PIN.

### 2.6 Native lock fallback

- On failed/dismissed auth → lock the Mac.
- Options, in order of preference:
  1. `pmset displaysleepnow` then rely on "require password immediately"
     (clean, no private API). **Recommend this for MVP.**
  2. Private framework `login` / `SACLockScreenImmediate` (cleaner UX but
     private API — avoid if you ever want Mac App Store; fine for Homebrew).
  3. The `osascript` Cmd+Ctrl+Q keystroke trick (works, slightly hacky).
- After triggering lock → set state to `Frozen`, tear down overlay windows,
  remove event monitors.

### 2.7 Prevent sleep while armed

- IOKit power assertion: `IOPMAssertionCreateWithName` with
  `kIOPMAssertionTypePreventUserIdleSystemSleep` (or display sleep).
- Release the assertion when disarmed/frozen.
- Crate: `io-kit-sys` or hand-rolled FFI. Make it a toggle in settings.

---

## 3. State machine

```
        ┌────────┐  user arms (menubar/CLI)   ┌────────┐
        │  Idle  │ ─────────────────────────► │ Armed  │
        └────────┘                            └────────┘
             ▲                                     │
             │ disarm                              │ app loses focus
             │                                     ▼
             │                              ┌────────────┐
             │                              │ Presenting │ (overlay up,
             │                              └────────────┘  awaiting auth)
             │                              auth ok │   │ auth fail/dismiss
             │   ◄──────────────────────────────────┘   │
             │       (back to Armed, overlay down)       ▼
             │                                     ┌──────────┐
             └─────────────────────────────────── │  Frozen  │
                       user clicks "Resume"        └──────────┘
                                                   (triggered native lock,
                                                    overlay torn down)
```

---

## 4. Settings (persisted as JSON in `~/Library/Application Support/veil/`)

MVP:

- `pin` (Argon2 hash, in Keychain not JSON)
- `recoveryCodeHash` (Keychain)
- `backgroundImagePath`
- `showClock` (bool, default true)
- `graceTimeoutMs` (default 0)
- `preventSleep` (bool, default true)
- `launchAtLogin` (bool)

Post-MVP (architect for, don't build yet):

- `backgroundType`: `image | video | gif | color`
- `hintText`
- `clockStyle` / position
- per-display background
- themes registry (JSON + assets in app support dir, user can add)

---

## 5. Frontend screens (React + TS + Vite)

1. **First-run setup** — set PIN (twice), generate + display recovery code,
   pick a background image, done.
2. **Overlay** — fullscreen; background image; centered clock + date
   (style it close to the native macOS lock screen — large light clock,
   date above). Click/keypress → reveals auth.
3. **Auth prompt** — Touch ID auto-fires; PIN pad fallback; "use recovery code"
   link.
4. **Settings window** — edit PIN (requires current PIN/Touch ID), change
   background, toggle clock/sleep/launch-at-login, view/regenerate recovery code,
   set grace timeout.

---

## 6. Distribution

### 6.1 Build & sign

- `cargo tauri build` → produces `.app` + `.dmg`.
- **Code sign** with Developer ID Application cert.
- **Notarize** via `notarytool` (required on Sequoia or Gatekeeper blocks it).
- Staple the ticket.
- Entitlements: this app needs to observe input + run an overlay; keep
  entitlements minimal. No hardened-runtime conflicts expected for
  LocalAuthentication / Keychain.

### 6.2 Homebrew cask

- Publish `.dmg` to a GitHub Releases page.
- Write a `Cask` formula (`Casks/veil.rb`) pointing at the release artifact +
  sha256. Host in your own tap repo: `homebrew-veil` (so:
  `brew install loa212/veil/veil`).

### 6.3 CLI

- Tiny binary (or shell shim) installed alongside the app:
  - `veil` → launch the .app
  - `veil arm` → arm immediately (IPC to running app, or launch+arm)
  - `veil status` → print state (Idle/Armed/Frozen)
- Simplest impl: CLI talks to the running app over a local unix socket or
  Tauri's single-instance/IPC. For MVP, `veil` just opens the app and that's
  enough.

### 6.4 CI (GitHub Actions, runs on `macos-14`+ runner)

- Build, sign, notarize on tag push.
- Attach `.dmg` to the release.
- Auto-bump the cask sha256 / version in the tap repo.
- (This is the _only_ place "reproducible builds" lives — no Docker.)

---

## 7. Build order (suggested for Claude Code, smallest-risk-first)

1. **Spike the natives separately** (throwaway Rust bins):
   - a) detect focus loss + log it
   - b) spawn a borderless screen-saver-level window covering all displays
   - c) fire a Touch ID prompt and read the result
   - d) trigger native lock via `pmset displaysleepnow`
     Confirm each works on your Sequoia machine before integrating.
2. Scaffold Tauri app + tray (arm/disarm/status) + state machine.
3. Wire focus-loss → overlay present (no auth yet, just show/hide).
4. Add auth: Touch ID, then PIN pad, then recovery code.
5. Add Keychain storage + first-run setup flow.
6. Add lock-on-fail → Frozen → Resume.
7. Add settings window + persistence.
8. Add prevent-sleep assertion + launch-at-login.
9. Polish overlay (clock/date styling).
10. Sign, notarize, .dmg, cask, CI, CLI shim.

---

## 8. Known risks / things to verify early

- **Overlay can't fully "lock you out" of your own machine** — it's a soft
  layer; a determined attacker with physical access could force-quit via
  some path. That's _why_ the real macOS lock is the fallback. Set
  expectations: this is anti-snooping, not anti-forensics.
- **Focus-loss loops**: presenting the overlay must not itself re-trigger the
  watcher. The `Presenting` state flag guards this.
- **Tauri window level**: confirm you can set `NSScreenSaverWindowLevel` on the
  underlying NSWindow via raw handle in Tauri 2.x — this is the linchpin of
  the "covers everything" behavior.
- **Notarization on Sequoia** is non-optional for smooth `brew` install.
- **Private API** (`SACLockScreenImmediate`) — avoid unless `pmset` UX is
  unacceptable; it bars you from MAS and risks breakage across OS updates.

---

## 9. Stretch / post-MVP (don't build now, keep modular)

- Animated backgrounds (GIF / MP4) — `<video>`/`<img>` in the overlay webview.
- Per-display backgrounds.
- Hint text ("back in 5") visible without auth.
- Clock styling options.
- "Any input arms auth" via global event monitor (vs click-to-reveal).
- Auto-arm on idle timer.
- ping via TG or HTTP if you want to get fancy with presence detection + take pictures of snoopers with the webcam.
- Themes registry: JSON file + assets folder in app support dir, user can add custom themes with different clock styles, backgrounds, and hint text, displayed on a website and installable with a CLI command that copies assets and updates the JSON.
