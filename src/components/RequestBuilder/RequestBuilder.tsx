import { useState, type FormEvent } from 'react'
import { sendHttpRequest } from '@/lib'
import type { HttpMethod, HttpResponse, HttpError } from '@/types'
import styles from './RequestBuilder.module.css'

const HTTP_METHODS: HttpMethod[] = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']

export interface RequestBuilderProps {
  initialUrl?: string
  initialMethod?: HttpMethod
}

export function RequestBuilder({
  initialUrl = 'https://httpbin.org/get',
  initialMethod = 'GET',
}: RequestBuilderProps) {
  const [method, setMethod] = useState<HttpMethod>(initialMethod)
  const [url, setUrl] = useState<string>(initialUrl)
  const [body, setBody] = useState<string>('')
  const [showBody, setShowBody] = useState<boolean>(false)
  const [activeTab, setActiveTab] = useState<'body' | 'headers'>('body')

  const [loading, setLoading] = useState<boolean>(false)
  const [response, setResponse] = useState<HttpResponse | null>(null)
  const [error, setError] = useState<HttpError | null>(null)

  const handleSubmit = async (e?: FormEvent) => {
    if (e) {
      e.preventDefault()
    }
    const cleanUrl = url.trim()
    if (!cleanUrl || loading) return

    setLoading(true)
    setError(null)
    setResponse(null)

    try {
      const res = await sendHttpRequest({
        method,
        url: cleanUrl,
        body: showBody && body.trim() ? body : undefined,
      })
      setResponse(res)
    } catch (err: unknown) {
      if (typeof err === 'object' && err !== null && 'kind' in err && 'message' in err) {
        setError(err as HttpError)
      } else {
        setError({
          kind: 'network_failure',
          message: err instanceof Error ? err.message : String(err),
        })
      }
    } finally {
      setLoading(false)
    }
  }

  const getStatusClass = (status: number) => {
    if (status >= 200 && status < 300) return styles.statusSuccess
    if (status >= 300 && status < 400) return styles.statusRedirect
    if (status >= 400 && status < 500) return styles.statusClientError
    return styles.statusServerError
  }

  return (
    <div className={styles.container}>
      <form onSubmit={handleSubmit} className={styles.requestBar}>
        <select
          value={method}
          onChange={(e) => {
            const nextMethod = e.target.value as HttpMethod
            setMethod(nextMethod)
            if (['POST', 'PUT', 'PATCH'].includes(nextMethod)) {
              setShowBody(true)
            }
          }}
          className={styles.methodSelect}
          aria-label="HTTP Method"
        >
          {HTTP_METHODS.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </select>

        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="httpbin.org/get or http://localhost:3000"
          className={styles.urlInput}
          aria-label="Request URL"
        />

        <button type="submit" disabled={loading || !url.trim()} className={styles.sendButton}>
          {loading ? 'Sending...' : 'Send'}
        </button>
      </form>

      <div className={styles.bodyToggleBar}>
        <button
          type="button"
          onClick={() => setShowBody(!showBody)}
          className={styles.toggleBodyButton}
        >
          {showBody ? 'Hide Request Body' : 'Add Request Body'}
        </button>
      </div>

      {showBody && (
        <div className={styles.bodySection}>
          <label htmlFor="request-body" className={styles.bodyLabel}>
            Request Body
          </label>
          <textarea
            id="request-body"
            value={body}
            onChange={(e) => setBody(e.target.value)}
            placeholder="JSON or raw text payload..."
            className={styles.bodyTextarea}
            rows={4}
          />
        </div>
      )}

      {error && (
        <div className={styles.errorCard} role="alert">
          <div className={styles.errorTitle}>
            <span>Request Failed</span>
            <span className={styles.errorKindBadge}>{error.kind}</span>
          </div>
          <p className={styles.errorMessage}>{error.message}</p>
        </div>
      )}

      {response && (
        <section className={styles.responseSection} aria-label="Response Details">
          <div className={styles.responseHeader}>
            <div className={styles.responseMetrics}>
              <span className={`${styles.statusBadge} ${getStatusClass(response.status)}`}>
                {response.status}
              </span>
              <span className={styles.metricItem}>
                Time: <strong className={styles.metricValue}>{response.elapsed_ms} ms</strong>
              </span>
              <span className={styles.metricItem}>
                Size: <strong className={styles.metricValue}>{response.size_bytes} B</strong>
              </span>
              {response.is_binary && <span className={styles.binaryBadge}>Binary (Base64)</span>}
            </div>

            <div className={styles.tabGroup}>
              <button
                type="button"
                onClick={() => setActiveTab('body')}
                className={`${styles.tabButton} ${activeTab === 'body' ? styles.tabButtonActive : ''}`}
              >
                Body
              </button>
              <button
                type="button"
                onClick={() => setActiveTab('headers')}
                className={`${styles.tabButton} ${activeTab === 'headers' ? styles.tabButtonActive : ''}`}
              >
                Headers ({response.headers.length})
              </button>
            </div>
          </div>

          <div className={styles.responseBodyContainer}>
            {activeTab === 'body' ? (
              <pre className={styles.responseBody}>
                <code>{response.body || '<Empty response body>'}</code>
              </pre>
            ) : (
              <div className={styles.headersList}>
                {response.headers.map((h, i) => (
                  <div key={`${h.name}-${i}`} className={styles.headerRow}>
                    <span className={styles.headerName}>{h.name}:</span>
                    <span className={styles.headerValue}>{h.value}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>
      )}
    </div>
  )
}
