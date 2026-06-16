# Veil

A macOS "soft lockscreen": a menubar (accessory) app. On an explicit user action
(menubar "Lock now" or the Cmd+Ctrl+L global hotkey) it drops a fullscreen
overlay across every display and prompts for Touch ID / PIN / recovery code. On a
successful unlock it returns to Idle (it does NOT re-lock on focus loss); on
repeated auth failure it triggers the native macOS lock and freezes. Tauri 2.x
(Rust core) + React 19 + TS + Vite. State machine: `Idle | Presenting | Frozen`.

The build is structured in phases — see [PLAN.md](PLAN.md) for the spec and
`~/.claude/plans/we-need-to-implement-eventual-lightning.md` for the
implementation plan and phase order.

## Commands

- `bun run tauri dev` — run the app (menubar only; no dock icon, no auto-window)
- `bun run check:all` — full quality gate (typecheck, eslint, prettier, cargo
  fmt/clippy, vitest, cargo test). Run after significant changes.
- `bun run fix:all` — auto-fix lint/format (JS + Rust)

## Conventions

- **Commands register in ONE place**: `generate_handler!` in
  [src-tauri/src/lib.rs](src-tauri/src/lib.rs). There is no web/dispatch layer.
- **Serde naming**: persisted structs (e.g. `Settings`) use snake_case; the
  matching TS interface must match field-for-field. Command arg/return structs
  that need camelCase use `#[serde(rename_all = "camelCase")]`.
- **State is the source of truth in Rust**: `app.emit("state-changed", …)` →
  frontend `listen`. UI calls back via `invoke`. Each window is its own JS
  context; they stay in sync through events.
- **Zustand**: read via `useStore.getState()` in listeners/callbacks to avoid
  render cascades; guard no-op `set()`s.

## Entry points

- [src-tauri/src/lib.rs](src-tauri/src/lib.rs) — app wiring: plugins, managed
  state, tray, global lock hotkey, first-run gate, command registration,
  `ActivationPolicy::Accessory`, `prevent_exit` so it lives in the tray.
- [src-tauri/src/state.rs](src-tauri/src/state.rs) — the state machine
  (`Idle | Presenting | Frozen`); all transitions flow through `transition()`.
  Lock is manual (no focus watcher): `Idle → Presenting` via "Lock now" / hotkey.
- [src-tauri/src/commands.rs](src-tauri/src/commands.rs) — the entire Tauri
  command surface (`lock_now`, auth, settings, …).
- [src-tauri/src/overlay/](src-tauri/src/overlay/) — multi-display overlay:
  spawn one window per monitor at its logical geometry, elevate each to
  screen-saver window level + activate the app via the raw `NSWindow` handle
  (`nswindow.rs`) so all displays cover at once.
- [src-tauri/src/auth/](src-tauri/src/auth/) — Touch ID (`touchid.rs`, dispatched
  on the main thread) + argon2 PIN/recovery hashing (`pin.rs`). The global lock
  hotkey is registered in [src-tauri/src/lib.rs](src-tauri/src/lib.rs).
- [src-tauri/src/lock.rs](src-tauri/src/lock.rs) — native lock via the private
  `SACLockScreenImmediate` (login.framework, dlopen/dlsym) — no permission.
- [src-tauri/src/screen.rs](src-tauri/src/screen.rs) — observes
  `com.apple.screenIsUnlocked` to auto-clear `Frozen → Idle` after the user logs
  back into macOS.
- [src/main.tsx](src/main.tsx) — picks the view per window
  (overlay / settings / first-run) via [src/lib/window.ts](src/lib/window.ts).
- [src/lib/commands.ts](src/lib/commands.ts) + [src/lib/ipc.ts](src/lib/ipc.ts)
  — typed wrappers over the Rust command surface.

## Maintaining entry points

When you add a new top-level module/subsystem or move a key file, update the
**Entry points** list above (add/rename/remove the one-line pointer). Keep it a
curated short list of "start here" files — not an exhaustive index. Leave the
rest of this file alone unless a command or convention actually changed.
