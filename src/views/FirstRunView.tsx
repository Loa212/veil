import { useEffect, useRef, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { RecoveryCodeDisplay } from '@/components/RecoveryCodeDisplay'
import { generateRecovery, setupPin } from '@/lib/commands'

type Step = 'pin' | 'recovery'

const PIN_MIN = 4

/**
 * First-run setup: choose a PIN, then save a one-time recovery code. Closing the
 * window after this leaves Veil idle in the menubar, ready to lock.
 */
export function FirstRunView() {
  const [step, setStep] = useState<Step>('pin')
  const [pin, setPin] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [recovery, setRecovery] = useState('')
  const pinRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (step === 'pin') pinRef.current?.focus()
  }, [step])

  const submitPin = async () => {
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
      const code = await generateRecovery()
      setRecovery(code)
      setStep('recovery')
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(false)
    }
  }

  const finish = () => {
    void getCurrentWindow().close()
  }

  if (step === 'recovery') {
    return (
      <div className="mx-auto flex min-h-screen max-w-md flex-col justify-center gap-6 p-8">
        <div>
          <h1 className="text-2xl font-semibold">Save your recovery code</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            This is the only time it’s shown. Store it somewhere safe — it’s the
            only way back in if you forget your PIN.
          </p>
        </div>

        <RecoveryCodeDisplay code={recovery} />

        <div className="rounded-md border border-amber-500/40 bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-300">
          Heads up: if you enter the wrong PIN too many times, Veil locks your
          Mac to the macOS login screen. You only set this up once.
        </div>

        <button
          type="button"
          onClick={finish}
          className="mt-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        >
          I’ve saved it — finish
        </button>
      </div>
    )
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
          className="rounded-md border border-input bg-transparent px-3 py-2 text-center text-lg tracking-widest focus:outline-none focus:ring-2 focus:ring-ring"
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
            if (e.key === 'Enter') void submitPin()
          }}
          placeholder="Confirm PIN"
          className="rounded-md border border-input bg-transparent px-3 py-2 text-center text-lg tracking-widest focus:outline-none focus:ring-2 focus:ring-ring"
        />
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <button
        type="button"
        disabled={busy}
        onClick={() => void submitPin()}
        className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
      >
        Continue
      </button>
    </div>
  )
}
