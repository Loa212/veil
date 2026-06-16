# Veil

A macOS "soft lockscreen": a menubar (accessory) app that drops a fullscreen
overlay across every display the instant the app loses focus while _Armed_,
prompts for Touch ID / PIN / recovery code, and falls back to the native macOS
lock screen on failure. Tauri 2.x (Rust core) + React 19 + TS + Vite.

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
  state, tray, focus watcher, command registration, `ActivationPolicy::Accessory`.
- [src-tauri/src/state.rs](src-tauri/src/state.rs) — the state machine
  (`Idle | Armed | Presenting | Frozen`); all transitions flow through
  `transition()`.
- [src-tauri/src/commands.rs](src-tauri/src/commands.rs) — the entire Tauri
  command surface.
- [src-tauri/src/overlay/](src-tauri/src/overlay/) — multi-display overlay:
  spawn one window per `NSScreen`, elevate each to screen-saver window level via
  the raw `NSWindow` handle (`nswindow.rs`).
- [src-tauri/src/focus.rs](src-tauri/src/focus.rs) — `NSApplicationDidResignActive`
  watcher; arms the overlay on the `Armed → Presenting` edge only.
- [src-tauri/src/auth/](src-tauri/src/auth/) — Touch ID (`touchid.rs`) + argon2
  PIN/recovery hashing (`pin.rs`).
- [src/main.tsx](src/main.tsx) — picks the view per window
  (overlay / settings / first-run) via [src/lib/window.ts](src/lib/window.ts).
- [src/lib/commands.ts](src/lib/commands.ts) + [src/lib/ipc.ts](src/lib/ipc.ts)
  — typed wrappers over the Rust command surface.

## Maintaining entry points

When you add a new top-level module/subsystem or move a key file, update the
**Entry points** list above (add/rename/remove the one-line pointer). Keep it a
curated short list of "start here" files — not an exhaustive index. Leave the
rest of this file alone unless a command or convention actually changed.
