import { useEffect, useState } from 'react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { AuthPrompt } from '@/components/AuthPrompt'
import { Clock } from '@/components/Clock'
import { useAppState } from '@/hooks/useAppState'
import { loadSettings } from '@/lib/commands'
import { defaultSettings, type Settings } from '@/types/settings'

interface OverlayViewProps {
  isPrimary: boolean
  index: number
}

/**
 * Fullscreen lock-screen overlay: background image (or dark fallback) + clock,
 * native-lock-screen styled. Any input on the primary display (key, mouse move,
 * or click) reveals the auth prompt; secondary displays show only the backdrop.
 */
export function OverlayView({ isPrimary, index }: OverlayViewProps) {
  useAppState()
  const [revealed, setRevealed] = useState(false)
  const [initialDigit, setInitialDigit] = useState<string | undefined>()
  const [settings, setSettings] = useState<Settings>(defaultSettings)

  useEffect(() => {
    loadSettings()
      .then(setSettings)
      .catch(() => undefined)
  }, [])

  // Reveal on any user input while still on the backdrop (primary only).
  useEffect(() => {
    if (!isPrimary || revealed) return

    const reveal = (digit?: string) => {
      setInitialDigit(digit)
      setRevealed(true)
    }
    const onKey = (e: KeyboardEvent) => {
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

  const bgUrl = settings.background_image_path
    ? convertFileSrc(settings.background_image_path)
    : null

  return (
    <div className="relative flex h-screen w-screen items-center justify-center overflow-hidden bg-neutral-950 text-white select-none">
      {/* Background image + dark scrim for legibility. */}
      {bgUrl && (
        <img
          src={bgUrl}
          alt=""
          className="absolute inset-0 h-full w-full object-cover"
        />
      )}
      <div className="absolute inset-0 bg-black/40" />

      {/* Foreground content. */}
      <div className="relative flex flex-col items-center gap-16">
        {settings.show_clock && <Clock />}

        {revealed && isPrimary ? (
          <AuthPrompt isPrimary={isPrimary} initialDigit={initialDigit} />
        ) : (
          <p className="text-sm text-white/40">
            {isPrimary ? 'press any key to unlock' : `display ${index}`}
          </p>
        )}
      </div>
    </div>
  )
}
