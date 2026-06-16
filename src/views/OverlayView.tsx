import { useState } from 'react'
import { AuthPrompt } from '@/components/AuthPrompt'
import { useAppState } from '@/hooks/useAppState'

interface OverlayViewProps {
  isPrimary: boolean
  index: number
}

/**
 * Fullscreen lock-screen overlay. The first interaction on the primary display
 * reveals the auth prompt (Touch ID auto-fires; PIN / recovery fallback).
 * Secondary displays only show the backdrop — auth lives on the primary.
 */
export function OverlayView({ isPrimary, index }: OverlayViewProps) {
  useAppState()
  const [revealed, setRevealed] = useState(false)

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-neutral-950 text-white select-none">
      {revealed && isPrimary ? (
        <AuthPrompt isPrimary={isPrimary} />
      ) : isPrimary ? (
        // Click-anywhere surface to reveal the prompt. A full-bleed button keeps
        // it keyboard-accessible without wrapping the (button-containing) prompt.
        <button
          type="button"
          onClick={() => setRevealed(true)}
          className="flex h-full w-full items-center justify-center text-center"
        >
          <div>
            <p className="text-3xl font-light tracking-wide">Veil</p>
            <p className="mt-3 text-sm text-white/40">click to unlock</p>
          </div>
        </button>
      ) : (
        <div className="text-center">
          <p className="text-3xl font-light tracking-wide">Veil</p>
          <p className="mt-3 text-sm text-white/40">display {index}</p>
        </div>
      )}
    </div>
  )
}
