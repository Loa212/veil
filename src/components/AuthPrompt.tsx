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
// Forgiving budget: legit users sometimes fumble (spamming Space to wake Touch
// ID, a stray key while it's laggy), so we allow several wrong attempts before
// dropping to the macOS lock.
const MAX_ATTEMPTS = 5

// Innocent keys are forgiven, but not infinitely: mashing the spacebar a bunch
// isn't legit, so past this many forgiven presses we lock too.
const MAX_FORGIVEN = 15

// Keys that are innocent "wake/navigation" presses — ignored entirely so they
// neither pollute the PIN nor count against the user.
const FORGIVEN_KEYS = new Set([
  ' ',
  'Enter',
  'Escape',
  'Tab',
  'Shift',
  'Control',
  'Alt',
  'Meta',
  'CapsLock',
  'ArrowUp',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'Home',
  'End',
  'PageUp',
  'PageDown',
])

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
  const forgivenCount = useRef(0)

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

  // Append a character to the PIN buffer (digits from the pad, or any typed key
  // that isn't forgiven — non-digits naturally produce a wrong PIN and fail).
  const pushChar = useCallback(
    (c: string) => {
      if (busy) return
      setError(false)
      setPin(prev => {
        const next = (prev + c).slice(0, PIN_LENGTH)
        if (next.length === PIN_LENGTH) void submitPin(next)
        return next
      })
    },
    [busy, submitPin]
  )

  const onDelete = useCallback(() => setPin(prev => prev.slice(0, -1)), [])

  // Physical-keyboard handling. Digits enter the PIN; Backspace deletes;
  // innocent wake/navigation keys are ignored; anything else (letters, symbols —
  // e.g. blind-typing behind the overlay) lands in the PIN, so it fails into the
  // macOS lock after the forgiving attempt budget.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Backspace') {
        onDelete()
        return
      }
      if (FORGIVEN_KEYS.has(e.key)) {
        // Forgiven, but capped: a few wake/nav presses are fine; mashing isn't.
        forgivenCount.current += 1
        if (forgivenCount.current > MAX_FORGIVEN) void fallbackToMacLock()
        return
      }
      // Single printable character (digit, letter, or symbol).
      if (e.key.length === 1) pushChar(e.key)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [pushChar, onDelete])

  return (
    <div className="flex flex-col items-center gap-7">
      <PinPad
        length={pin.length}
        maxLength={PIN_LENGTH}
        disabled={busy}
        error={error}
        onDigit={pushChar}
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
