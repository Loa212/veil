import { QueryClient } from '@tanstack/react-query'

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Don't refetch on window focus in desktop app
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 1000 * 60 * 5,
      gcTime: 1000 * 60 * 10,
    },
    mutations: {
      retry: 1,
    },
  },
})
