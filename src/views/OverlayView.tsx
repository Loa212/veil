import { useAppState } from '@/hooks/useAppState'

interface OverlayViewProps {
  isPrimary: boolean
  index: number
}

/**
 * Fullscreen lock-screen overlay. Phase 1 placeholder — clock, background, and
 * the auth prompt land in later phases.
 */
export function OverlayView({ isPrimary, index }: OverlayViewProps) {
  useAppState()

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-black/90 text-white select-none">
      <div className="text-center">
        <p className="text-2xl font-light tracking-wide">Veil</p>
        <p className="mt-2 text-sm text-white/50">
          overlay {index}
          {isPrimary ? ' · primary' : ''}
        </p>
      </div>
    </div>
  )
}
