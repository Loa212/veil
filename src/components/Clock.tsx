import { useEffect, useState } from 'react'

/** Large, light clock + date styled like the native macOS lock screen. */
export function Clock() {
  const [now, setNow] = useState(() => new Date())

  useEffect(() => {
    // Tick on the minute boundary, then every minute.
    const id = setInterval(() => setNow(new Date()), 1000)
    return () => clearInterval(id)
  }, [])

  const time = now.toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
  const date = now.toLocaleDateString(undefined, {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
  })

  return (
    <div className="flex flex-col items-center text-white select-none">
      <p className="text-lg font-medium tracking-wide text-white/80">{date}</p>
      <p className="mt-1 text-[8rem] leading-none font-thin tracking-tight tabular-nums">
        {time}
      </p>
    </div>
  )
}
