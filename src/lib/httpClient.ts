import { invoke } from '@tauri-apps/api/core'
import type { HttpRequest, HttpResponse, HttpError } from '@/types'

/**
 * Sends an HTTP request through the Rust backend via Tauri IPC.
 *
 * @param request The request configuration (method, url, headers, body, timeout_ms).
 * @returns Resolves with the typed HttpResponse, or rejects with a typed HttpError.
 */
export async function sendHttpRequest(request: HttpRequest): Promise<HttpResponse> {
  try {
    return await invoke<HttpResponse>('send_http_request', { request })
  } catch (error: unknown) {
    if (typeof error === 'object' && error !== null && 'kind' in error && 'message' in error) {
      throw error as HttpError
    }

    const message =
      typeof error === 'string'
        ? error
        : error instanceof Error
          ? error.message
          : JSON.stringify(error)

    const fallbackError: HttpError = {
      kind: 'network_failure',
      message,
    }
    throw fallbackError
  }
}
