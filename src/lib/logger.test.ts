import { describe, it, expect, vi } from 'vitest'
import { logger } from '@/lib/logger'

describe('logger', () => {
  it('exposes debug, info, warn, error methods', () => {
    expect(typeof logger.debug).toBe('function')
    expect(typeof logger.info).toBe('function')
    expect(typeof logger.warn).toBe('function')
    expect(typeof logger.error).toBe('function')
  })

  it('logger.warn calls console.warn', () => {
    const spy = vi.spyOn(console, 'warn').mockImplementation(() => {})
    logger.warn('test warning')
    expect(spy).toHaveBeenCalledWith('[tweak:warn]', 'test warning')
    spy.mockRestore()
  })

  it('logger.error calls console.error', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    logger.error('test error')
    expect(spy).toHaveBeenCalledWith('[tweak:error]', 'test error')
    spy.mockRestore()
  })
})

