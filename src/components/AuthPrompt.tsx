import { useCallback, useEffect, useRef, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { PinPad } from './PinPad'
import {
  authenticateTouchId,
  fallbackToMacLock,
  verifyPin,
} from '@/lib/commands'

/**
 * Re-key the overlay window. The Touch ID system sheet steals key/focus and
 * doesn't return it on dismiss, leaving the PIN keyboard listener dead — so we
 * explicitly refocus after the biometric prompt resolves.
 */
async function refocusOverlay() {
  try {
    await getCurrentWindow().setFocus()
  } catch {
    /* not under Tauri (dev) */
  }
  window.focus()
}

/** Fire Touch ID and refocus the overlay afterwards (success closes it anyway). */
async function tryTouchId() {
  await authenticateTouchId().catch(() => undefined)
  await refocusOverlay()
}

const PIN_LENGTH = 4
const MAX_ATTEMPTS = 3

interface AuthPromptProps {
  /** Only the primary display auto-fires Touch ID and shows the full prompt. */
  isPrimary: boolean
  /** A digit that triggered the reveal, seeded as the first PIN entry. */
  initialDigit?: string
}

/**
 * Auth prompt shown when the overlay is interacted with. Touch ID auto-fires on
 * the primary display; PIN is the fallback. Too many failures (or the explicit
 * "macOS lock screen" link) drops to the native macOS lock via `auth_failed()`.
 */
export function AuthPrompt({ isPrimary, initialDigit }: AuthPromptProps) {
  const [pin, setPin] = useState(
    initialDigit && initialDigit >= '0' && initialDigit <= '9'
      ? initialDigit
      : ''
  )
  const [error, setError] = useState(false)
  const [busy, setBusy] = useState(false)
  const [attempts, setAttempts] = useState(0)
  const touchIdTried = useRef(false)

  // Auto-fire Touch ID once on mount (primary display only). On success the
  // Rust side tears the overlay down; otherwise we fall through to the PIN pad.
  useEffect(() => {
    if (!isPrimary || touchIdTried.current) return
    touchIdTried.current = true
    void tryTouchId()
  }, [isPrimary])

  const submitPin = useCallback(
    async (value: string) => {
      setBusy(true)
      const ok = await verifyPin(value).catch(() => false)
      setBusy(false)
      if (ok) return // overlay closes via state-changed
      const next = attempts + 1
      setAttempts(next)
      setError(true)
      setPin('')
      if (next >= MAX_ATTEMPTS) void fallbackToMacLock()
      else setTimeout(() => setError(false), 600)
    },
    [attempts]
  )

  const onDigit = useCallback(
    (d: string) => {
      if (busy) return
      setError(false)
      setPin(prev => {
        const next = (prev + d).slice(0, PIN_LENGTH)
        if (next.length === PIN_LENGTH) void submitPin(next)
        return next
      })
    },
    [busy, submitPin]
  )

  const onDelete = useCallback(() => setPin(prev => prev.slice(0, -1)), [])

  // Physical-keyboard support for the PIN pad.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key >= '0' && e.key <= '9') onDigit(e.key)
      else if (e.key === 'Backspace') onDelete()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onDigit, onDelete])

  return (
    <div className="flex flex-col items-center gap-7">
      <PinPad
        length={pin.length}
        maxLength={PIN_LENGTH}
        disabled={busy}
        error={error}
        onDigit={onDigit}
        onDelete={onDelete}
      />
      <div className="flex flex-col items-center gap-2 text-xs text-white/50">
        {isPrimary && (
          <button
            type="button"
            onClick={() => void tryTouchId()}
            className="hover:text-white/80"
          >
            Use Touch ID
          </button>
        )}
        <button
          type="button"
          onClick={() => void fallbackToMacLock()}
          className="hover:text-white/80"
        >
          macOS lock screen
        </button>
      </div>
    </div>
  )
}
