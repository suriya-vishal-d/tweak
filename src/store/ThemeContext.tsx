import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import type { ResolvedTheme, ThemeSetting } from '@/types'

// ---- Storage key ----
const STORAGE_KEY = 'tweak:theme'

// ---- Context shape ----
interface ThemeContextValue {
  /** The user's chosen setting (light | dark | system) */
  setting: ThemeSetting
  /** The currently applied theme after resolving 'system' */
  resolved: ResolvedTheme
  /** Set the theme setting */
  setTheme: (theme: ThemeSetting) => void
  /** Cycle: light → dark → system */
  cycleTheme: () => void
}

const ThemeContext = createContext<ThemeContextValue | null>(null)

// ---- Helpers ----
function getSystemPreference(): ResolvedTheme {
  if (typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark'
  }
  return 'light'
}

function loadStoredSetting(): ThemeSetting {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored === 'light' || stored === 'dark' || stored === 'system') {
      return stored
    }
  } catch {
    // localStorage not available (e.g., SSR or privacy mode)
  }
  return 'system'
}

function applyThemeClass(resolved: ResolvedTheme): void {
  const html = document.documentElement
  html.classList.remove('theme-light', 'theme-dark')
  html.classList.add(`theme-${resolved}`)
}

const CYCLE_ORDER: ThemeSetting[] = ['light', 'dark', 'system']

// ---- Provider ----
interface ThemeProviderProps {
  children: ReactNode
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const [setting, setSetting] = useState<ThemeSetting>(loadStoredSetting)
  const [systemPref, setSystemPref] = useState<ResolvedTheme>(getSystemPreference)

  // Listen for system preference changes
  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const handler = (e: MediaQueryListEvent) => {
      setSystemPref(e.matches ? 'dark' : 'light')
    }
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [])

  const resolved: ResolvedTheme = setting === 'system' ? systemPref : setting

  // Apply class to <html> whenever resolved theme changes
  useEffect(() => {
    applyThemeClass(resolved)
  }, [resolved])

  const setTheme = useCallback((next: ThemeSetting) => {
    setSetting(next)
    try {
      localStorage.setItem(STORAGE_KEY, next)
    } catch {
      // ignore
    }
  }, [])

  const cycleTheme = useCallback(() => {
    setSetting((prev) => {
      const idx = CYCLE_ORDER.indexOf(prev)
      const next = CYCLE_ORDER[(idx + 1) % CYCLE_ORDER.length]
      try {
        localStorage.setItem(STORAGE_KEY, next)
      } catch {
        // ignore
      }
      return next
    })
  }, [])

  const value = useMemo<ThemeContextValue>(
    () => ({ setting, resolved, setTheme, cycleTheme }),
    [setting, resolved, setTheme, cycleTheme],
  )

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
}

// ---- Hook ----
export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext)
  if (!ctx) {
    throw new Error('useTheme must be used within a ThemeProvider')
  }
  return ctx
}

