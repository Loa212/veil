import { invoke, invokeVoid } from './ipc'
import type { AppState, AuthOutcome } from '@/types/state'
import type { Settings } from '@/types/settings'

/**
 * Typed wrappers over the Tauri command surface (src-tauri/src/commands.rs).
 * Each command is registered exactly once in `generate_handler!` — there is no
 * second registration site.
 */

// ── State machine ──────────────────────────────────────────────────────────
export const lockNow = () => invokeVoid('lock_now')
export const getState = () => invoke<AppState>('get_state')
export const resume = () => invokeVoid('resume')

// ── Auth ───────────────────────────────────────────────────────────────────
export const authenticateTouchId = () =>
  invoke<AuthOutcome>('authenticate_touchid')
export const verifyPin = (pin: string) => invoke<boolean>('verify_pin', { pin })
/** Deliberately drop to the macOS lock screen (the fallback). */
export const fallbackToMacLock = () => invokeVoid('auth_failed')

// ── PIN setup ────────────────────────────────────────────────────────────────
export const isPinConfigured = () => invoke<boolean>('is_pin_configured')
export const setupPin = (pin: string) => invokeVoid('setup_pin', { pin })
/** Change the PIN; authorize with the current PIN and/or a prior Touch ID. */
export const changePin = (
  newPin: string,
  opts: { currentPin?: string; touchIdOk?: boolean }
) => invoke<boolean>('change_pin', { newPin, ...opts })

// ── Settings ─────────────────────────────────────────────────────────────────
export const loadSettings = () => invoke<Settings>('load_settings')
export const saveSettings = (settings: Settings) =>
  invokeVoid('save_settings', { settings })
export const pickBackground = () => invoke<string | null>('pick_background')
export const setLaunchAtLogin = (enabled: boolean) =>
  invokeVoid('set_launch_at_login', { enabled })

// ── Windows ──────────────────────────────────────────────────────────────────
export const openSettingsWindow = () => invokeVoid('open_settings_window')

// ── Updates ──────────────────────────────────────────────────────────────────
export const checkForUpdates = () => invokeVoid('check_for_updates')
