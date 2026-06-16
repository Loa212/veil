import { useAppState } from '@/hooks/useAppState'
import { useVeilStore } from '@/store/veil-store'

/**
 * Settings window. Phase 1 placeholder — real controls (PIN edit, background
 * picker, toggles) land in Phase 7.
 */
export function SettingsView() {
  useAppState()
  const appState = useVeilStore(s => s.appState)

  return (
    <div className="min-h-screen p-8">
      <h1 className="text-xl font-semibold">Veil — Settings</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        Current state: <span className="font-mono">{appState}</span>
      </p>
    </div>
  )
}
