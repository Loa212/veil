import { create } from 'zustand'
import type { AppState, AuthStage } from '@/types/state'
import type { Settings } from '@/types/settings'

interface VeilStore {
  /** Mirror of the Rust state machine, kept in sync via the `state-changed` event. */
  appState: AppState
  authStage: AuthStage
  pinError: string | null
  settings: Settings | null

  setAppState: (s: AppState) => void
  setAuthStage: (s: AuthStage) => void
  setPinError: (e: string | null) => void
  setSettings: (s: Settings) => void
}

export const useVeilStore = create<VeilStore>(set => ({
  appState: 'idle',
  authStage: 'idle',
  pinError: null,
  settings: null,

  setAppState: s =>
    set(state => (state.appState === s ? state : { appState: s })),
  setAuthStage: s =>
    set(state => (state.authStage === s ? state : { authStage: s })),
  setPinError: e =>
    set(state => (state.pinError === e ? state : { pinError: e })),
  setSettings: s => set({ settings: s }),
}))
