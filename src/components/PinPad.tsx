import { cn } from '@/lib/utils'

interface PinPadProps {
  /** Current entered length, for the dot indicator. */
  length: number
  minLength: number
  disabled?: boolean
  error?: boolean
  onDigit: (d: string) => void
  onDelete: () => void
  onSubmit: () => void
}

// Fixed keypad layout: `submit` is bottom-left and `del` is backspace.
const KEYS = ['1', '2', '3', '4', '5', '6', '7', '8', '9', 'submit', '0', 'del']

const MAX_VISIBLE_DOTS = 12
const DOTS = Array.from({ length: MAX_VISIBLE_DOTS }, (_, i) => `dot-${i}`)

/** macOS-lock-screen-style numeric PIN pad with a dot indicator. */
export function PinPad({
  length,
  minLength,
  disabled,
  error,
  onDigit,
  onDelete,
  onSubmit,
}: PinPadProps) {
  const visibleDots = Math.max(minLength, Math.min(length, MAX_VISIBLE_DOTS))

  return (
    <div
      className={cn('flex flex-col items-center gap-7', error && 'veil-shake')}
    >
      <div className="flex gap-3">
        {DOTS.slice(0, visibleDots).map((id, i) => (
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
          if (key === 'submit') {
            return (
              <button
                key={cellId}
                type="button"
                disabled={disabled || length < minLength}
                onClick={onSubmit}
                className="flex h-16 w-16 items-center justify-center rounded-full text-xs font-medium text-white/70 transition-colors hover:bg-white/10 disabled:opacity-30"
              >
                Unlock
              </button>
            )
          }
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
              disabled={disabled}
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
