import { getCurrentWindow } from '@tauri-apps/api/window'

export type WindowRole = 'overlay' | 'settings' | 'first-run'

export interface WindowContext {
  role: WindowRole
  /** Display index for overlay windows (0 = primary). */
  index: number
  /** Whether this overlay is on the primary display (where auth lives). */
  isPrimary: boolean
}

/**
 * Resolve which view a window should render. Overlay windows carry their role
 * and display info in the URL query string (set by overlay/windows.rs); the
 * settings and first-run windows are identified by their Tauri window label.
 */
export function resolveWindowContext(): WindowContext {
  const params = new URLSearchParams(window.location.search)
  const role = params.get('role')

  if (role === 'overlay') {
    return {
      role: 'overlay',
      index: Number(params.get('idx') ?? 0),
      isPrimary: params.get('primary') === 'true',
    }
  }

  const label = getCurrentWindow().label
  if (label === 'first-run') {
    return { role: 'first-run', index: 0, isPrimary: true }
  }

  // Default: settings window (also the fallback for plain `bun run dev`).
  return { role: 'settings', index: 0, isPrimary: true }
}
