import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { useAppState } from '@/hooks/useAppState'
import { useSaveSettings, useSettings } from '@/hooks/useSettings'
import { authenticateTouchId, changePin } from '@/lib/commands'
import type { Settings } from '@/types/settings'

export function SettingsView() {
  useAppState()
  const { data: settings } = useSettings()

  return (
    <div className="mx-auto max-w-lg space-y-10 p-8">
      <h1 className="text-2xl font-semibold">Veil — Settings</h1>
      <ChangePinSection />
      {settings && <BehaviorSection settings={settings} />}
      <HotkeySection />
    </div>
  )
}

// ── Change PIN ────────────────────────────────────────────────────────────────

function ChangePinSection() {
  const [currentPin, setCurrentPin] = useState('')
  const [newPin, setNewPin] = useState('')
  const [confirm, setConfirm] = useState('')
  const [status, setStatus] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  const reset = () => {
    setCurrentPin('')
    setNewPin('')
    setConfirm('')
  }

  const submit = async (touchIdOk: boolean) => {
    setStatus(null)
    if (newPin.length < 4 || !/^\d+$/.test(newPin)) {
      setStatus('New PIN must be at least 4 digits.')
      return
    }
    if (newPin !== confirm) {
      setStatus('New PINs do not match.')
      return
    }
    setBusy(true)
    try {
      const ok = await changePin(newPin, {
        currentPin: touchIdOk ? undefined : currentPin,
        touchIdOk,
      })
      if (ok) {
        setStatus('PIN updated.')
        reset()
      } else {
        setStatus('Authorization failed — wrong current PIN.')
      }
    } catch (e) {
      setStatus(String(e))
    } finally {
      setBusy(false)
    }
  }

  const authorizeWithTouchId = async () => {
    const outcome = await authenticateTouchId().catch(() => 'failure')
    if (outcome === 'success') void submit(true)
    else setStatus('Touch ID was not confirmed.')
  }

  return (
    <Section
      title="Change PIN"
      description="Authorize with your current PIN or Touch ID, then set a new one."
    >
      <div className="space-y-3">
        <LabeledInput
          label="Current PIN"
          type="password"
          value={currentPin}
          onChange={v => setCurrentPin(v.replace(/\D/g, ''))}
        />
        <LabeledInput
          label="New PIN"
          type="password"
          value={newPin}
          onChange={v => setNewPin(v.replace(/\D/g, ''))}
        />
        <LabeledInput
          label="Confirm new PIN"
          type="password"
          value={confirm}
          onChange={v => setConfirm(v.replace(/\D/g, ''))}
        />
        <div className="flex items-center gap-3 pt-1">
          <Button disabled={busy} onClick={() => void submit(false)}>
            Update PIN
          </Button>
          <Button
            variant="outline"
            disabled={busy}
            onClick={() => void authorizeWithTouchId()}
          >
            Authorize with Touch ID
          </Button>
        </div>
        {status && <p className="text-sm text-muted-foreground">{status}</p>}
      </div>
    </Section>
  )
}

// ── Behavior toggles ──────────────────────────────────────────────────────────

function BehaviorSection({ settings }: { settings: Settings }) {
  const save = useSaveSettings()

  // The query cache (updated optimistically by the mutation) is the source of
  // truth, so no local mirror is needed.
  const update = (patch: Partial<Settings>) => {
    save.mutate({ ...settings, ...patch })
  }

  return (
    <Section title="Behavior">
      <div className="space-y-4">
        <ToggleRow
          label="Show clock on the lock overlay"
          checked={settings.show_clock}
          onChange={v => update({ show_clock: v })}
        />
        <ToggleRow
          label="Prevent the Mac from sleeping while locked"
          checked={settings.prevent_sleep}
          onChange={v => update({ prevent_sleep: v })}
        />
        <ToggleRow
          label="Launch Veil at login"
          checked={settings.launch_at_login}
          onChange={v => update({ launch_at_login: v })}
        />
        <div className="flex items-center justify-between">
          <span className="text-sm">Grace delay before locking (ms)</span>
          <input
            type="number"
            min={0}
            step={250}
            value={settings.grace_timeout_ms}
            onChange={e =>
              update({ grace_timeout_ms: Math.max(0, Number(e.target.value)) })
            }
            className="w-24 rounded-md border border-input bg-transparent px-2 py-1 text-right text-sm focus:ring-2 focus:ring-ring focus:outline-none"
          />
        </div>
      </div>
    </Section>
  )
}

function HotkeySection() {
  return (
    <Section title="Lock hotkey">
      <p className="text-sm text-muted-foreground">
        Press{' '}
        <kbd className="rounded border border-input bg-muted px-1.5 py-0.5 font-mono text-xs">
          ⌘⌃L
        </kbd>{' '}
        anywhere to lock. (Customization coming soon.)
      </p>
    </Section>
  )
}

// ── Small presentational helpers ──────────────────────────────────────────────

function Section({
  title,
  description,
  children,
}: {
  title: string
  description?: string
  children: React.ReactNode
}) {
  return (
    <section className="space-y-3">
      <div>
        <h2 className="text-sm font-semibold tracking-wide text-foreground/90 uppercase">
          {title}
        </h2>
        {description && (
          <p className="mt-1 text-sm text-muted-foreground">{description}</p>
        )}
      </div>
      {children}
    </section>
  )
}

function ToggleRow({
  label,
  checked,
  onChange,
}: {
  label: string
  checked: boolean
  onChange: (v: boolean) => void
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-sm">{label}</span>
      <Switch checked={checked} onCheckedChange={onChange} />
    </div>
  )
}

function LabeledInput({
  label,
  value,
  onChange,
  type = 'text',
}: {
  label: string
  value: string
  onChange: (v: string) => void
  type?: string
}) {
  return (
    <label className="flex items-center justify-between gap-4">
      <span className="text-sm">{label}</span>
      <input
        type={type}
        inputMode="numeric"
        value={value}
        onChange={e => onChange(e.target.value)}
        className="w-40 rounded-md border border-input bg-transparent px-3 py-1.5 text-center tracking-widest focus:ring-2 focus:ring-ring focus:outline-none"
      />
    </label>
  )
}
