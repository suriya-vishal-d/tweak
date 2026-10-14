import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { ThemeProvider, useTheme } from '@/store'
import type { ReactNode } from 'react'

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => {
      store[key] = value
    },
    removeItem: (key: string) => {
      delete store[key]
    },
    clear: () => {
      store = {}
    },
  }
})()

Object.defineProperty(globalThis, 'localStorage', {
  value: localStorageMock,
  writable: true,
})

// Mock matchMedia
Object.defineProperty(globalThis, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: query === '(prefers-color-scheme: dark)' ? false : false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

function wrapper({ children }: { children: ReactNode }) {
  return <ThemeProvider>{children}</ThemeProvider>
}

describe('useTheme', () => {
  beforeEach(() => {
    localStorageMock.clear()
    document.documentElement.className = ''
  })

  it('defaults to system theme', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })
    expect(result.current.setting).toBe('system')
  })

  it('resolves system theme to light when prefers-color-scheme is light', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })
    expect(result.current.resolved).toBe('light')
  })

  it('setTheme changes the setting', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('dark')
    })

    expect(result.current.setting).toBe('dark')
    expect(result.current.resolved).toBe('dark')
  })

  it('setTheme persists to localStorage', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('light')
    })

    expect(localStorageMock.getItem('tweak:theme')).toBe('light')
  })

  it('cycleTheme cycles light → dark → system → light', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    // Start at system (default)
    expect(result.current.setting).toBe('system')

    act(() => { result.current.cycleTheme() })
    expect(result.current.setting).toBe('light')

    act(() => { result.current.cycleTheme() })
    expect(result.current.setting).toBe('dark')

    act(() => { result.current.cycleTheme() })
    expect(result.current.setting).toBe('system')
  })

  it('applies theme-light class to document.documentElement for light setting', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('light')
    })

    expect(document.documentElement.classList.contains('theme-light')).toBe(true)
    expect(document.documentElement.classList.contains('theme-dark')).toBe(false)
  })

  it('applies theme-dark class to document.documentElement for dark setting', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('dark')
    })

    expect(document.documentElement.classList.contains('theme-dark')).toBe(true)
    expect(document.documentElement.classList.contains('theme-light')).toBe(false)
  })

  it('throws when useTheme is used outside ThemeProvider', () => {
    // suppress the expected error output
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    expect(() => renderHook(() => useTheme())).toThrow(
      'useTheme must be used within a ThemeProvider',
    )
    consoleSpy.mockRestore()
  })
})

