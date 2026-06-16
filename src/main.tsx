import ReactDOM from 'react-dom/client'
import { QueryClientProvider } from '@tanstack/react-query'
import { queryClient } from '@/lib/query-client'
import { resolveWindowContext } from '@/lib/window'
import { OverlayView } from '@/views/OverlayView'
import { SettingsView } from '@/views/SettingsView'
import { FirstRunView } from '@/views/FirstRunView'
import '@/styles.css'

const ctx = resolveWindowContext()

// Overlay windows are transparent at the OS level — mark <html> so styles.css
// keeps the document background clear while OverlayView paints its own.
if (ctx.role === 'overlay') {
  document.documentElement.classList.add('overlay')
}

function Root() {
  switch (ctx.role) {
    case 'overlay':
      return <OverlayView isPrimary={ctx.isPrimary} index={ctx.index} />
    case 'first-run':
      return <FirstRunView />
    case 'settings':
      return <SettingsView />
  }
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <QueryClientProvider client={queryClient}>
    <Root />
  </QueryClientProvider>
)
