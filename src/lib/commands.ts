import { invoke, invokeVoid } from './ipc'
import type { AppState, AuthOutcome } from '@/types/state'
import type { Settings } from '@/types/settings'

/**
 * Typed wrappers over the Tauri command surface (src-tauri/src/commands.rs).
 * Each command is registered exactly once in `generate_handler!` — there is no
 * second registration site.
 */

// ── State machine ──────────────────────────────────────────────────────────
export const arm = () => invokeVoid('arm')
export const disarm = () => invokeVoid('disarm')
export const getState = () => invoke<AppState>('get_state')
export const resume = () => invokeVoid('resume')

// ── Overlay (normally driven by the focus watcher; exposed for dev/testing) ──
export const presentOverlay = () => invokeVoid('present_overlay')
export const dismissOverlay = () => invokeVoid('dismiss_overlay')

// ── Auth ───────────────────────────────────────────────────────────────────
export const authenticateTouchId = () =>
  invoke<AuthOutcome>('authenticate_touchid')
export const verifyPin = (pin: string) => invoke<boolean>('verify_pin', { pin })
export const verifyRecovery = (code: string) =>
  invoke<boolean>('verify_recovery', { code })
export const authFailed = () => invokeVoid('auth_failed')

// ── PIN / recovery setup ─────────────────────────────────────────────────────
export const isPinConfigured = () => invoke<boolean>('is_pin_configured')
export const setupPin = (pin: string) => invokeVoid('setup_pin', { pin })
export const changePin = (currentPin: string, newPin: string) =>
  invoke<boolean>('change_pin', { currentPin, newPin })
export const generateRecovery = () => invoke<string>('generate_recovery')
export const regenerateRecovery = () => invoke<string>('regenerate_recovery')

// ── Settings ─────────────────────────────────────────────────────────────────
export const loadSettings = () => invoke<Settings>('load_settings')
export const saveSettings = (settings: Settings) =>
  invokeVoid('save_settings', { settings })
export const pickBackground = () => invoke<string | null>('pick_background')
export const setLaunchAtLogin = (enabled: boolean) =>
  invokeVoid('set_launch_at_login', { enabled })

// ── Windows ──────────────────────────────────────────────────────────────────
export const openSettingsWindow = () => invokeVoid('open_settings_window')
