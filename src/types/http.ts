/**
 * Supported HTTP methods.
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS'

/**
 * A single HTTP header represented as a name-value pair.
 * Preserves duplicate headers (e.g. multiple Set-Cookie headers) and order.
 */
export interface HttpHeader {
  name: string
  value: string
}

/**
 * Parameters for sending an HTTP request through the Tauri Rust backend.
 */
export interface HttpRequest {
  method: HttpMethod | string
  url: string
  headers?: Record<string, string>
  body?: string
  timeout_ms?: number
}

/**
 * Response received from the backend HTTP client.
 */
export interface HttpResponse {
  status: number
  headers: HttpHeader[]
  body: string
  is_binary: boolean
  elapsed_ms: number
  size_bytes: number
}

/**
 * Categorized error types returned by the backend.
 */
export type HttpErrorKind =
  'invalid_url' | 'network_failure' | 'timeout' | 'invalid_method' | 'invalid_header'

/**
 * Typed, serializable error received from the backend on request failure.
 */
export interface HttpError {
  kind: HttpErrorKind
  message: string
}
