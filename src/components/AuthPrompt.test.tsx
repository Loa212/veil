import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { AuthPrompt } from './AuthPrompt'
import {
  authenticateTouchId,
  fallbackToMacLock,
  verifyPin,
} from '@/lib/commands'

vi.mock('@/lib/commands', () => ({
  authenticateTouchId: vi.fn(),
  fallbackToMacLock: vi.fn(),
  verifyPin: vi.fn(),
}))

describe('AuthPrompt', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(authenticateTouchId).mockResolvedValue('unavailable')
    vi.mocked(fallbackToMacLock).mockResolvedValue(undefined)
    vi.mocked(verifyPin).mockResolvedValue(true)
  })

  it('submits a PIN longer than four digits', async () => {
    render(<AuthPrompt isPrimary={false} />)

    const pin = '12345678901234567890'
    for (const digit of pin) {
      fireEvent.click(screen.getByRole('button', { name: digit }))
    }
    expect(verifyPin).not.toHaveBeenCalled()

    fireEvent.click(screen.getByRole('button', { name: 'Unlock' }))
    await waitFor(() => expect(verifyPin).toHaveBeenCalledWith(pin))
  })

  it('submits the entered PIN with the Enter key', async () => {
    render(<AuthPrompt isPrimary={false} />)

    for (const digit of ['9', '8', '7', '6']) {
      fireEvent.keyDown(window, { key: digit })
    }
    expect(verifyPin).not.toHaveBeenCalled()

    fireEvent.keyDown(window, { key: 'Enter' })
    await waitFor(() => expect(verifyPin).toHaveBeenCalledWith('9876'))
  })

  it('allows only one verification while a submission is pending', async () => {
    let resolveVerification: ((value: boolean) => void) | undefined
    vi.mocked(verifyPin).mockImplementation(
      () =>
        new Promise(resolve => {
          resolveVerification = resolve
        })
    )
    render(<AuthPrompt isPrimary={false} />)

    for (const digit of ['1', '2', '3', '4']) {
      fireEvent.click(screen.getByRole('button', { name: digit }))
    }
    const unlock = screen.getByRole('button', { name: 'Unlock' })
    fireEvent.click(unlock)
    fireEvent.keyDown(window, { key: 'Enter' })

    expect(verifyPin).toHaveBeenCalledTimes(1)
    resolveVerification?.(true)
    await waitFor(() => expect(unlock).not.toBeDisabled())
  })
})
