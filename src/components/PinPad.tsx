import { cn } from '@/lib/utils'

interface PinPadProps {
  /** Current entered length, for the dot indicator. */
  length: number
  maxLength: number
  disabled?: boolean
  error?: boolean
  onDigit: (d: string) => void
  onDelete: () => void
}

// Fixed keypad layout: '' is the empty bottom-left cell, 'del' the backspace.
const KEYS = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '', '0', 'del']

const DOTS = Array.from({ length: 8 }, (_, i) => `dot-${i}`)

/** macOS-lock-screen-style numeric PIN pad with a dot indicator. */
export function PinPad({
  length,
  maxLength,
  disabled,
  error,
  onDigit,
  onDelete,
}: PinPadProps) {
  return (
    <div
      className={cn('flex flex-col items-center gap-7', error && 'veil-shake')}
    >
      <div className="flex gap-3">
        {DOTS.slice(0, maxLength).map((id, i) => (
          <span
            key={id}
            className={cn(
              'h-3 w-3 rounded-full border transition-colors',
              error
                ? 'border-red-500 bg-red-500'
                : cn('border-white/50', i < length && 'bg-white')
            )}
          />
        ))}
      </div>

      <div className="grid grid-cols-3 gap-4">
        {KEYS.map((key, i) => {
          const cellId = `cell-${i}`
          if (key === '') return <span key={cellId} />
          if (key === 'del') {
            return (
              <button
                key={cellId}
                type="button"
                disabled={disabled || length === 0}
                onClick={onDelete}
                className="flex h-16 w-16 items-center justify-center rounded-full text-sm text-white/70 transition-colors hover:bg-white/10 disabled:opacity-30"
              >
                ⌫
              </button>
            )
          }
          return (
            <button
              key={cellId}
              type="button"
              disabled={disabled || length >= maxLength}
              onClick={() => onDigit(key)}
              className="flex h-16 w-16 items-center justify-center rounded-full bg-white/5 text-2xl font-light text-white transition-colors hover:bg-white/15 disabled:opacity-30"
            >
              {key}
            </button>
          )
        })}
      </div>
    </div>
  )
}
