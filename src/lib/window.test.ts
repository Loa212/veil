import { describe, it, expect, beforeEach, vi } from 'vitest'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { resolveWindowContext } from './window'

function setSearch(search: string) {
  Object.defineProperty(window, 'location', {
    writable: true,
    value: { search },
  })
}

describe('resolveWindowContext', () => {
  beforeEach(() => {
    setSearch('')
    vi.mocked(getCurrentWindow).mockReturnValue({
      label: 'settings',
    } as ReturnType<typeof getCurrentWindow>)
  })

  it('routes overlay windows from the query string', () => {
    setSearch('?role=overlay&idx=2&primary=true')
    expect(resolveWindowContext()).toEqual({
      role: 'overlay',
      index: 2,
      isPrimary: true,
    })
  })

  it('marks non-primary overlays', () => {
    setSearch('?role=overlay&idx=1&primary=false')
    expect(resolveWindowContext()).toMatchObject({
      role: 'overlay',
      index: 1,
      isPrimary: false,
    })
  })

  it('routes the first-run window by label', () => {
    vi.mocked(getCurrentWindow).mockReturnValue({
      label: 'first-run',
    } as ReturnType<typeof getCurrentWindow>)
    expect(resolveWindowContext().role).toBe('first-run')
  })

  it('defaults to the settings window', () => {
    expect(resolveWindowContext().role).toBe('settings')
  })
})
