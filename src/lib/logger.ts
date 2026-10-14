/**
 * logger.ts — thin wrapper around console for application logging.
 * In production builds, only warnings and errors are emitted.
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error'

const IS_DEV = import.meta.env.DEV

function shouldLog(level: LogLevel): boolean {
  if (IS_DEV) return true
  return level === 'warn' || level === 'error'
}

export const logger = {
  debug: (...args: unknown[]) => {
    if (shouldLog('debug')) {
      console.warn('[tweak:debug]', ...args)
    }
  },
  info: (...args: unknown[]) => {
    if (shouldLog('info')) {
      console.warn('[tweak:info]', ...args)
    }
  },
  warn: (...args: unknown[]) => {
    if (shouldLog('warn')) {
      console.warn('[tweak:warn]', ...args)
    }
  },
  error: (...args: unknown[]) => {
    if (shouldLog('error')) {
      console.error('[tweak:error]', ...args)
    }
  },
}
