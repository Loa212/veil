import { useEffect, useState } from 'react'
import { AuthPrompt } from '@/components/AuthPrompt'
import { useAppState } from '@/hooks/useAppState'

interface OverlayViewProps {
  isPrimary: boolean
  index: number
}

/**
 * Fullscreen lock-screen overlay. Any input on the primary display (key, mouse
 * move, or click) reveals the auth prompt (Touch ID auto-fires; PIN / recovery
 * fallback). Secondary displays only show the backdrop — auth lives on primary.
 */
export function OverlayView({ isPrimary, index }: OverlayViewProps) {
  useAppState()
  const [revealed, setRevealed] = useState(false)
  // If the reveal was triggered by typing a digit, forward it so the first
  // keystroke isn't lost.
  const [initialDigit, setInitialDigit] = useState<string | undefined>()

  // Reveal on any user input while still on the backdrop.
  useEffect(() => {
    if (!isPrimary || revealed) return

    const reveal = (digit?: string) => {
      setInitialDigit(digit)
      setRevealed(true)
    }
    const onKey = (e: KeyboardEvent) => {
      // Ignore lone modifier presses.
      if (['Shift', 'Control', 'Alt', 'Meta'].includes(e.key)) return
      reveal(e.key >= '0' && e.key <= '9' ? e.key : undefined)
    }
    const onPointer = () => reveal()

    window.addEventListener('keydown', onKey)
    window.addEventListener('mousemove', onPointer)
    window.addEventListener('pointerdown', onPointer)
    return () => {
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('mousemove', onPointer)
      window.removeEventListener('pointerdown', onPointer)
    }
  }, [isPrimary, revealed])

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-neutral-950 text-white select-none">
      {revealed && isPrimary ? (
        <AuthPrompt isPrimary={isPrimary} initialDigit={initialDigit} />
      ) : (
        <div className="text-center">
          <p className="text-3xl font-light tracking-wide">Veil</p>
          <p className="mt-3 text-sm text-white/40">
            {isPrimary ? 'press any key to unlock' : `display ${index}`}
          </p>
        </div>
      )}
    </div>
  )
}
