import { useEffect } from 'react'
import { listen } from '@/lib/ipc'
import { getState } from '@/lib/commands'
import { useVeilStore } from '@/store/veil-store'
import type { AppState } from '@/types/state'

/**
 * Keeps this window's `appState` in sync with the Rust state machine: hydrates
 * once on mount, then follows every `state-changed` event. Rust is the single
 * source of truth; each window subscribes independently.
 */
export function useAppState() {
  useEffect(() => {
    let unlisten: (() => void) | undefined
    let cancelled = false

    getState()
      .then(s => useVeilStore.getState().setAppState(s))
      .catch(() => {
        /* not running under Tauri (plain vite dev) — ignore */
      })

    listen<AppState>('state-changed', e => {
      useVeilStore.getState().setAppState(e.payload)
    }).then(fn => {
      if (cancelled) fn()
      else unlisten = fn
    })

    return () => {
      cancelled = true
      unlisten?.()
    }
  }, [])
}
