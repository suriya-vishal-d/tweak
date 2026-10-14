import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { RequestBuilder } from './RequestBuilder'
import * as lib from '@/lib'
import type { HttpResponse, HttpError } from '@/types'

vi.mock('@/lib', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib')>()
  return {
    ...actual,
    sendHttpRequest: vi.fn(),
  }
})

describe('RequestBuilder component', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders method select, url input, and send button with default values', () => {
    render(<RequestBuilder initialUrl="https://httpbin.org/get" initialMethod="GET" />)

    const methodSelect = screen.getByRole('combobox', { name: /http method/i })
    expect(methodSelect).toHaveValue('GET')

    const urlInput = screen.getByRole('textbox', { name: /request url/i })
    expect(urlInput).toHaveValue('https://httpbin.org/get')

    const sendButton = screen.getByRole('button', { name: /send/i })
    expect(sendButton).toBeEnabled()
  })

  it('disables send button when URL is cleared', async () => {
    const user = userEvent.setup()
    render(<RequestBuilder initialUrl="https://httpbin.org/get" />)

    const urlInput = screen.getByRole('textbox', { name: /request url/i })
    await user.clear(urlInput)

    const sendButton = screen.getByRole('button', { name: /send/i })
    expect(sendButton).toBeDisabled()
  })

  it('submits request and renders response details on success', async () => {
    const user = userEvent.setup()
    const mockResponse: HttpResponse = {
      status: 200,
      headers: [{ name: 'content-type', value: 'application/json' }],
      body: '{"success":true}',
      is_binary: false,
      elapsed_ms: 120,
      size_bytes: 16,
    }

    vi.mocked(lib.sendHttpRequest).mockResolvedValueOnce(mockResponse)

    render(<RequestBuilder initialUrl="https://httpbin.org/get" initialMethod="GET" />)

    const sendButton = screen.getByRole('button', { name: /send/i })
    await user.click(sendButton)

    await waitFor(() => {
      expect(lib.sendHttpRequest).toHaveBeenCalledWith({
        method: 'GET',
        url: 'https://httpbin.org/get',
        body: undefined,
      })
    })

    expect(await screen.findByText('200')).toBeInTheDocument()
    expect(screen.getByText('120 ms')).toBeInTheDocument()
    expect(screen.getByText('16 B')).toBeInTheDocument()
    expect(screen.getByText('{"success":true}')).toBeInTheDocument()
  })

  it('displays typed error card on request failure', async () => {
    const user = userEvent.setup()
    const mockError: HttpError = {
      kind: 'timeout',
      message: 'Connection timed out after 5000ms',
    }

    vi.mocked(lib.sendHttpRequest).mockRejectedValueOnce(mockError)

    render(<RequestBuilder initialUrl="https://httpbin.org/delay/10" initialMethod="GET" />)

    const sendButton = screen.getByRole('button', { name: /send/i })
    await user.click(sendButton)

    const errorAlert = await screen.findByRole('alert')
    expect(errorAlert).toBeInTheDocument()
    expect(screen.getByText('timeout')).toBeInTheDocument()
    expect(screen.getByText('Connection timed out after 5000ms')).toBeInTheDocument()
  })

  it('allows adding and sending a request body for POST method', async () => {
    const user = userEvent.setup()
    const mockResponse: HttpResponse = {
      status: 201,
      headers: [],
      body: 'Created',
      is_binary: false,
      elapsed_ms: 85,
      size_bytes: 7,
    }

    vi.mocked(lib.sendHttpRequest).mockResolvedValueOnce(mockResponse)

    render(<RequestBuilder initialUrl="https://httpbin.org/post" initialMethod="POST" />)

    const toggleBodyBtn = screen.getByRole('button', {
      name: /hide request body|add request body/i,
    })
    if (toggleBodyBtn.textContent?.includes('Add')) {
      await user.click(toggleBodyBtn)
    }

    const bodyTextarea = await screen.findByRole('textbox', { name: /request body/i })
    fireEvent.change(bodyTextarea, { target: { value: '{"title":"Test"}' } })

    const sendButton = screen.getByRole('button', { name: /send/i })
    await user.click(sendButton)

    await waitFor(() => {
      expect(lib.sendHttpRequest).toHaveBeenCalledWith({
        method: 'POST',
        url: 'https://httpbin.org/post',
        body: '{"title":"Test"}',
      })
    })

    expect(await screen.findByText('201')).toBeInTheDocument()
  })

  it('renders duplicate headers in the headers tab', async () => {
    const user = userEvent.setup()
    const mockResponse: HttpResponse = {
      status: 200,
      headers: [
        { name: 'set-cookie', value: 'session=123' },
        { name: 'set-cookie', value: 'theme=dark' },
      ],
      body: 'OK',
      is_binary: false,
      elapsed_ms: 50,
      size_bytes: 2,
    }

    vi.mocked(lib.sendHttpRequest).mockResolvedValueOnce(mockResponse)

    render(<RequestBuilder initialUrl="https://httpbin.org/cookies" initialMethod="GET" />)

    const sendButton = screen.getByRole('button', { name: /send/i })
    await user.click(sendButton)

    const headersTab = await screen.findByRole('button', { name: /headers \(2\)/i })
    await user.click(headersTab)

    expect(screen.getByText('session=123')).toBeInTheDocument()
    expect(screen.getByText('theme=dark')).toBeInTheDocument()
  })
})
