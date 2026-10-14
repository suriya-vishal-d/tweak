import { describe, it, expect, vi, beforeEach } from 'vitest'
import { sendHttpRequest } from './httpClient'
import type { HttpRequest, HttpResponse, HttpError } from '@/types'

const mockInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}))

describe('sendHttpRequest', () => {
  beforeEach(() => {
    mockInvoke.mockReset()
  })

  it('successfully invokes backend command and returns response', async () => {
    const mockResponse: HttpResponse = {
      status: 200,
      headers: [{ name: 'content-type', value: 'application/json' }],
      body: '{"message":"ok"}',
      is_binary: false,
      elapsed_ms: 42,
      size_bytes: 18,
    }
    mockInvoke.mockResolvedValueOnce(mockResponse)

    const request: HttpRequest = {
      method: 'GET',
      url: 'https://example.com/api',
    }

    const result = await sendHttpRequest(request)

    expect(mockInvoke).toHaveBeenCalledWith('send_http_request', { request })
    expect(result).toEqual(mockResponse)
  })

  it('propagates typed HttpError when backend rejects with typed error', async () => {
    const typedError: HttpError = {
      kind: 'timeout',
      message: 'Request timed out after 5000ms',
    }
    mockInvoke.mockRejectedValueOnce(typedError)

    const request: HttpRequest = {
      method: 'POST',
      url: 'https://example.com/slow',
      timeout_ms: 5000,
    }

    await expect(sendHttpRequest(request)).rejects.toEqual(typedError)
  })

  it('normalizes string errors into HttpError network_failure', async () => {
    mockInvoke.mockRejectedValueOnce('Connection refused')

    const request: HttpRequest = {
      method: 'GET',
      url: 'http://127.0.0.1:9999',
    }

    await expect(sendHttpRequest(request)).rejects.toEqual({
      kind: 'network_failure',
      message: 'Connection refused',
    })
  })
})
