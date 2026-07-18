import { useEffect, useState } from 'react'
import { getVersion } from '@tauri-apps/api/app'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'
import { useAppState } from '@/hooks/useAppState'
import { useSaveSettings, useSettings } from '@/hooks/useSettings'
import {
  authenticateTouchId,
  changePin,
  checkForUpdates,
  pickBackground,
} from '@/lib/commands'
import type { Settings } from '@/types/settings'

type Pane = 'general' | 'pin' | 'about'
const PIN_MIN = 4

const NAV: { id: Pane; label: string }[] = [
  { id: 'general', label: 'General' },
  { id: 'pin', label: 'PIN' },
  { id: 'about', label: 'About' },
]

export function SettingsView() {
  useAppState()
  const { data: settings } = useSettings()
  const [pane, setPane] = useState<Pane>('general')

  return (
    <div className="flex h-screen">
      {/* Left nav */}
      <nav className="w-44 shrink-0 border-r border-border bg-muted/30 p-3">
        <p className="px-2 pt-1 pb-3 text-sm font-semibold">Veil</p>
        <ul className="space-y-1">
          {NAV.map(item => (
            <li key={item.id}>
              <button
                type="button"
                onClick={() => setPane(item.id)}
                className={cn(
                  'w-full rounded-md px-3 py-1.5 text-left text-sm transition-colors',
                  pane === item.id
                    ? 'bg-primary/10 font-medium text-foreground'
                    : 'text-muted-foreground hover:bg-accent hover:text-foreground'
                )}
              >
                {item.label}
              </button>
            </li>
          ))}
        </ul>
      </nav>

      {/* Content pane */}
      <div className="flex-1 overflow-y-auto p-8">
        {pane === 'general' && (
          <div className="max-w-lg space-y-10">
            {settings && <BehaviorSection settings={settings} />}
            <HotkeySection />
          </div>
        )}
        {pane === 'pin' && (
          <div className="max-w-lg">
            <ChangePinSection />
          </div>
        )}
        {pane === 'about' && (
          <div className="max-w-lg">
            <AboutSection />
          </div>
        )}
      </div>
    </div>
  )
}

function AboutSection() {
  const [version, setVersion] = useState<string | null>(null)

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(null))
  }, [])

  return (
    <Section title="About Veil">
      <div className="space-y-5 text-sm text-muted-foreground">
        <div className="flex items-center justify-between gap-4">
          <div>
            <p className="font-medium text-foreground">Veil</p>
            <p className="font-mono text-xs">
              {version ? `Version ${version}` : 'Version unavailable'}
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => void checkForUpdates()}
          >
            Check for updates
          </Button>
        </div>

        <p>
          Veil is a soft lockscreen. Press{' '}
          <kbd className="rounded border border-input bg-muted px-1.5 py-0.5 font-mono text-xs">
            ⌘⌃L
          </kbd>{' '}
          or use the menubar to drop a fullscreen overlay; unlock with Touch ID
          or your PIN.
        </p>
        <p>
          If you forget your PIN, fail the unlock — Veil drops to the macOS lock
          screen. Log back into your Mac, then change your PIN here.
        </p>
      </div>
    </Section>
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
    if (newPin.length < PIN_MIN || !/^\d+$/.test(newPin)) {
      setStatus(`New PIN must be at least ${PIN_MIN} digits.`)
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

  const chooseBackground = async () => {
    const path = await pickBackground().catch(() => null)
    if (path) update({ background_image_path: path })
  }

  return (
    <Section title="Behavior">
      <div className="space-y-4">
        <div className="flex items-center justify-between gap-4">
          <div className="min-w-0">
            <span className="text-sm">Background image</span>
            {settings.background_image_path && (
              <p className="truncate text-xs text-muted-foreground">
                {settings.background_image_path.split('/').pop()}
              </p>
            )}
          </div>
          <div className="flex shrink-0 gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => void chooseBackground()}
            >
              Choose…
            </Button>
            {settings.background_image_path && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => update({ background_image_path: null })}
              >
                Clear
              </Button>
            )}
          </div>
        </div>
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
