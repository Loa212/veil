import { useEffect, useRef, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { setupPin } from '@/lib/commands'

const PIN_MIN = 4

/**
 * First-run setup: choose a PIN. That's it — there's no recovery code; if the
 * PIN is forgotten, the macOS lock screen is the fallback (fail auth → macOS
 * lock → log into the Mac → change the PIN in Settings). Closing the window
 * leaves Veil idle in the menubar, ready to lock.
 */
export function FirstRunView() {
  const [pin, setPin] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const pinRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    pinRef.current?.focus()
  }, [])

  const submit = async () => {
    if (pin.length < PIN_MIN || !/^\d+$/.test(pin)) {
      setError(`PIN must be at least ${PIN_MIN} digits.`)
      return
    }
    if (pin !== confirm) {
      setError('PINs do not match.')
      return
    }
    setBusy(true)
    setError(null)
    try {
      await setupPin(pin)
      void getCurrentWindow().close()
    } catch (e) {
      setError(String(e))
      setBusy(false)
    }
  }

  return (
    <div className="mx-auto flex min-h-screen max-w-md flex-col justify-center gap-6 p-8">
      <div>
        <h1 className="text-2xl font-semibold">Welcome to Veil</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Choose a PIN. You’ll use it (or Touch ID) to unlock when Veil is
          locked.
        </p>
      </div>

      <div className="flex flex-col gap-3">
        <input
          ref={pinRef}
          type="password"
          inputMode="numeric"
          value={pin}
          onChange={e => {
            setError(null)
            setPin(e.target.value.replace(/\D/g, ''))
          }}
          placeholder="New PIN"
          className="rounded-md border border-input bg-transparent px-3 py-2 text-center text-lg tracking-widest focus:ring-2 focus:ring-ring focus:outline-none"
        />
        <input
          type="password"
          inputMode="numeric"
          value={confirm}
          onChange={e => {
            setError(null)
            setConfirm(e.target.value.replace(/\D/g, ''))
          }}
          onKeyDown={e => {
            if (e.key === 'Enter') void submit()
          }}
          placeholder="Confirm PIN"
          className="rounded-md border border-input bg-transparent px-3 py-2 text-center text-lg tracking-widest focus:ring-2 focus:ring-ring focus:outline-none"
        />
      </div>

      <div className="rounded-md border border-amber-500/40 bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-300">
        If you enter the wrong PIN too many times, Veil locks your Mac to the
        macOS login screen — that’s also how you recover if you forget your PIN.
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <button
        type="button"
        disabled={busy}
        onClick={() => void submit()}
        className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
      >
        Set PIN
      </button>
    </div>
  )
}
