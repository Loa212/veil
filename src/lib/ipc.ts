import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { listen as tauriListen, type UnlistenFn } from '@tauri-apps/api/event'

/**
 * Thin native-only IPC layer. Veil has no web transport (unlike the reference
 * app), so commands and events go straight through Tauri's core/event APIs.
 */
export function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  return tauriInvoke<T>(cmd, args)
}

/** Invoke a command whose Rust return type is `()` (no payload). */
export function invokeVoid(
  cmd: string,
  args?: Record<string, unknown>
): Promise<void> {
  return tauriInvoke<null>(cmd, args).then(() => undefined)
}

export function listen<T>(
  event: string,
  handler: (event: { payload: T }) => void
): Promise<UnlistenFn> {
  return tauriListen<T>(event, handler)
}
