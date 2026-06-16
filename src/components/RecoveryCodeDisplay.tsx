import { useState } from 'react'

interface RecoveryCodeDisplayProps {
  code: string
}

/** Shows a recovery code in a monospace box with a copy-to-clipboard button. */
export function RecoveryCodeDisplay({ code }: RecoveryCodeDisplayProps) {
  const [copied, setCopied] = useState(false)

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {
      /* clipboard unavailable */
    }
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="rounded-md border border-input bg-muted/50 px-4 py-4 text-center font-mono text-xl tracking-widest select-all">
        {code}
      </div>
      <button
        type="button"
        onClick={() => void copy()}
        className="self-end text-xs text-muted-foreground hover:text-foreground"
      >
        {copied ? 'Copied ✓' : 'Copy'}
      </button>
    </div>
  )
}
