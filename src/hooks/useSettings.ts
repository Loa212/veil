import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { loadSettings, saveSettings } from '@/lib/commands'
import { defaultSettings, type Settings } from '@/types/settings'

const KEY = ['settings'] as const

export function useSettings() {
  return useQuery({
    queryKey: KEY,
    queryFn: async () => {
      try {
        return await loadSettings()
      } catch {
        return defaultSettings
      }
    },
  })
}

export function useSaveSettings() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (settings: Settings) => saveSettings(settings),
    onSuccess: (_data, settings) => {
      qc.setQueryData(KEY, settings)
    },
  })
}
