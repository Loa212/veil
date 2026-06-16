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

const KEYS = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '', '0', 'del']

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
    <div className="flex flex-col items-center gap-7">
      <div className={cn('flex gap-3', error && 'animate-pulse')}>
        {Array.from({ length: maxLength }).map((_, i) => (
          <span
            key={i}
            className={cn(
              'h-3 w-3 rounded-full border border-white/50 transition-colors',
              i < length && 'bg-white',
              error && 'border-red-400'
            )}
          />
        ))}
      </div>

      <div className="grid grid-cols-3 gap-4">
        {KEYS.map((key, i) => {
          if (key === '') return <span key={i} />
          if (key === 'del') {
            return (
              <button
                key={i}
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
              key={i}
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
