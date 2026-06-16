import { useEffect } from 'react'
import { useAppState } from '@/hooks/useAppState'
import { dismissOverlay } from '@/lib/commands'

interface OverlayViewProps {
  isPrimary: boolean
  index: number
}

/**
 * Fullscreen lock-screen overlay.
 *
 * Phase 3: opaque full-bleed surface that proves the window covers the whole
 * display (menu bar + Dock included). Until auth lands in Phase 4, any click or
 * Escape dismisses the overlay (Presenting -> Armed) so it isn't a dead end.
 */
export function OverlayView({ isPrimary, index }: OverlayViewProps) {
  useAppState()

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') void dismissOverlay()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  return (
    <button
      type="button"
      onClick={() => void dismissOverlay()}
      className="flex h-screen w-screen cursor-default items-center justify-center bg-neutral-950 text-center text-white select-none"
    >
      <div>
        <p className="text-3xl font-light tracking-wide">Veil</p>
        <p className="mt-3 text-sm text-white/40">
          display {index}
          {isPrimary ? ' · primary' : ''}
        </p>
        <p className="mt-8 text-xs text-white/30">
          click or press Esc to dismiss (auth lands in Phase 4)
        </p>
      </div>
    </button>
  )
}
