use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Maximum response body size allowed (20 MiB) to prevent memory exhaustion.
pub const MAX_RESPONSE_SIZE_BYTES: usize = 20 * 1024 * 1024;

/// Standard maximum redirect count.
pub const MAX_REDIRECT_LIMIT: usize = 10;

/// A single HTTP header represented as a key-value pair.
/// Preserves duplicate headers (e.g., Set-Cookie) and header casing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

/// HTTP request parameters sent from the frontend or CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub timeout_ms: Option<u64>,
}

/// Typed HTTP response containing status, headers as pairs, body, binary flag, latency, and payload size.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<HttpHeader>,
    pub body: String,
    pub is_binary: bool,
    pub elapsed_ms: u64,
    pub size_bytes: usize,
}

/// Typed, serializable errors for HTTP execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum HttpError {
    InvalidUrl(String),
    NetworkFailure(String),
    Timeout(String),
    InvalidMethod(String),
    InvalidHeader(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(msg) => write!(f, "Invalid URL: {msg}"),
            Self::NetworkFailure(msg) => write!(f, "Network failure: {msg}"),
            Self::Timeout(msg) => write!(f, "Request timed out: {msg}"),
            Self::InvalidMethod(msg) => write!(f, "Invalid HTTP method: {msg}"),
            Self::InvalidHeader(msg) => write!(f, "Invalid HTTP header: {msg}"),
        }
    }
}

impl std::error::Error for HttpError {}

/// Convert a reqwest::Error into our strongly-typed HttpError without panicking.
fn map_reqwest_error(err: reqwest::Error) -> HttpError {
    if err.is_timeout() {
        HttpError::Timeout(err.to_string())
    } else if err.is_builder() {
        HttpError::InvalidUrl(err.to_string())
    } else {
        HttpError::NetworkFailure(err.to_string())
    }
}

/// Simple, zero-dependency standard Base64 encoder for binary response payloads.
pub fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);

        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Returns `true` if the host portion of a bare (no-scheme) URL string refers to a
/// local or private/non-public network address that is unlikely to have a TLS certificate.
///
/// Recognised patterns:
/// - `localhost` (exact, case-insensitive) and `*.localhost`
/// - Single-label hostnames with no dots (e.g. `myserver`, `devbox`)
/// - mDNS / local-network TLDs: `*.local`, `*.internal`, `*.lan`
/// - IPv6 loopback `::1`
/// - IPv4 loopback `127.0.0.0/8`
/// - IPv4 private: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`
/// - IPv4 CGNAT/shared: `100.64.0.0/10` (RFC 6598 -- also used by Tailscale)
/// - IPv4 link-local: `169.254.0.0/16` (RFC 3927)
fn is_local_host(bare: &str) -> bool {
    // Strip optional port suffix.
    // For IPv6 bracketed notation "[::1]:8080" strip the trailing ":port" first.
    let host_part = if bare.starts_with('[') {
        // Bracketed IPv6, e.g. "[::1]" or "[::1]:8080"
        match bare.find(']') {
            Some(close) => &bare[1..close],
            None => bare,
        }
    } else {
        // Strip port: take everything before the last ':' only if the port is all digits.
        match bare.rfind(':') {
            Some(colon) if bare[colon + 1..].chars().all(|c| c.is_ascii_digit()) => {
                &bare[..colon]
            }
            _ => bare,
        }
    };

    let host_lower = host_part.to_ascii_lowercase();

    // localhost / *.localhost
    if host_lower == "localhost" || host_lower.ends_with(".localhost") {
        return true;
    }

    // Single-label hostname (no dots) -- e.g. "myserver", "devbox"
    // These are internal names that will never have a public TLS cert.
    if !host_lower.contains('.') {
        return true;
    }

    // mDNS / local-network TLDs: *.local, *.internal, *.lan
    if host_lower.ends_with(".local")
        || host_lower.ends_with(".internal")
        || host_lower.ends_with(".lan")
    {
        return true;
    }

    // IPv6 loopback ::1
    if host_lower == "::1" {
        return true;
    }

    // IPv4: parse octets
    let octets: Vec<&str> = host_lower.split('.').collect();
    if octets.len() == 4 {
        let parsed: Option<[u8; 4]> = (|| {
            Some([
                octets[0].parse::<u8>().ok()?,
                octets[1].parse::<u8>().ok()?,
                octets[2].parse::<u8>().ok()?,
                octets[3].parse::<u8>().ok()?,
            ])
        })();
        if let Some([a, b, c, _d]) = parsed {
            // 127.0.0.0/8 -- loopback
            if a == 127 {
                return true;
            }
            // 10.0.0.0/8 -- private
            if a == 10 {
                return true;
            }
            // 172.16.0.0/12 (172.16 - 172.31) -- private
            if a == 172 && (16..=31).contains(&b) {
                return true;
            }
            // 192.168.0.0/16 -- private
            if a == 192 && b == 168 {
                return true;
            }
            // 100.64.0.0/10 -- shared address / CGNAT (RFC 6598); also used by Tailscale.
            // 100.64 - 100.127
            if a == 100 && (64..=127).contains(&b) {
                return true;
            }
            // 169.254.0.0/16 -- link-local (RFC 3927)
            if a == 169 && b == 254 {
                return true;
            }
            let _ = c;
        }
    }

    false
}

/// Normalizes a user-entered URL.
///
/// If the input already contains `"://"` it is treated as having an explicit scheme
/// and is returned trimmed.
///
/// If no `"://"` is present a scheme is inferred from the host:
/// - Local/private addresses get `"http://"` -- they rarely have TLS.
///   Covered: `localhost`, `*.localhost`, single-label hostnames, `*.local`,
///   `*.internal`, `*.lan`, `127.x`, `10.x`, `172.16-31.x`, `192.168.x`,
///   `100.64-127.x` (CGNAT/RFC 6598), `169.254.x` (link-local), `::1`.
/// - Every other host gets `"https://"` -- public hosts almost always have TLS.
///
/// Scheme validation (http/https only) and empty-URL rejection are performed in
/// `execute_request`, not here.
pub fn normalize_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.contains("://") {
        return trimmed.to_string();
    }
    // Strip a leading path so we inspect only the host[:port] part.
    let bare_host = trimmed.split('/').next().unwrap_or(trimmed);
    if is_local_host(bare_host) {
        format!("http://{trimmed}")
    } else {
        format!("https://{trimmed}")
    }
}

/// Executes an HTTP request asynchronously with reqwest.
///
/// Returns `Ok(HttpResponse)` on success, or `Err(HttpError)` on validation or network failure.
/// Contains no `unwrap()` calls in production code.
pub async fn execute_request(req: HttpRequest) -> Result<HttpResponse, HttpError> {
    if req.url.trim().is_empty() {
        return Err(HttpError::InvalidUrl("URL cannot be empty".to_string()));
    }

    // 1. Normalize and validate URL
    let normalized = normalize_url(&req.url);
    let parsed_url = reqwest::Url::parse(&normalized).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(HttpError::InvalidUrl(format!(
            "Unsupported scheme '{}'. Only 'http' and 'https' are supported.",
            parsed_url.scheme()
        )));
    }

    // 2. Validate HTTP method
    let method = req
        .method
        .trim()
        .to_uppercase()
        .parse::<reqwest::Method>()
        .map_err(|e| HttpError::InvalidMethod(e.to_string()))?;

    // 3. Configure timeout (default to 30,000ms if not specified)
    let timeout = Duration::from_millis(req.timeout_ms.unwrap_or(30_000));

    // 4. Build client with explicit redirect policy (10 max)
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECT_LIMIT))
        .build()
        .map_err(|e| HttpError::NetworkFailure(format!("Failed to build HTTP client: {e}")))?;

    let mut request_builder = client.request(method, parsed_url).timeout(timeout);

    // 5. Attach headers if present
    if let Some(headers) = req.headers {
        for (key, val) in headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| HttpError::InvalidHeader(format!("Invalid header name '{key}': {e}")))?;
            let header_value = reqwest::header::HeaderValue::from_str(&val)
                .map_err(|e| HttpError::InvalidHeader(format!("Invalid header value for '{key}': {e}")))?;
            request_builder = request_builder.header(header_name, header_value);
        }
    }

    // 6. Attach body if present
    if let Some(body) = req.body {
        request_builder = request_builder.body(body);
    }

    // 7. Execute request and measure elapsed time
    let start_instant = Instant::now();
    let response = request_builder.send().await.map_err(map_reqwest_error)?;

    let status = response.status().as_u16();

    // Check Content-Length against maximum size limit
    if let Some(content_length) = response.content_length() {
        if content_length > MAX_RESPONSE_SIZE_BYTES as u64 {
            return Err(HttpError::NetworkFailure(format!(
                "Response size ({content_length} bytes) exceeds maximum limit of {} MB",
                MAX_RESPONSE_SIZE_BYTES / (1024 * 1024)
            )));
        }
    }

    // Collect response headers as a list of name-value pairs to preserve duplicates (e.g. Set-Cookie)
    let mut response_headers = Vec::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            response_headers.push(HttpHeader {
                name: name.as_str().to_string(),
                value: value_str.to_string(),
            });
        }
    }

    // Read response body bytes
    let body_bytes = response.bytes().await.map_err(map_reqwest_error)?;
    if body_bytes.len() > MAX_RESPONSE_SIZE_BYTES {
        return Err(HttpError::NetworkFailure(format!(
            "Response size ({} bytes) exceeds maximum limit of {} MB",
            body_bytes.len(),
            MAX_RESPONSE_SIZE_BYTES / (1024 * 1024)
        )));
    }

    let elapsed_ms = start_instant.elapsed().as_millis() as u64;
    let size_bytes = body_bytes.len();

    // Check if the response body is valid UTF-8. If not, encode as base64 to prevent corruption.
    let (body, is_binary) = match std::str::from_utf8(&body_bytes) {
        Ok(valid_text) => (valid_text.to_string(), false),
        Err(_) => (encode_base64(&body_bytes), true),
    };

    Ok(HttpResponse {
        status,
        headers: response_headers,
        body,
        is_binary,
        elapsed_ms,
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_successful_get_request() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await.unwrap();
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, Tweak!";
            socket.write_all(response.as_bytes()).await.unwrap();
            socket.flush().await.unwrap();
        });

        let req = HttpRequest {
            method: "GET".to_string(),
            url: format!("127.0.0.1:{port}/hello"), // Auto-prefix http://
            headers: None,
            body: None,
            timeout_ms: Some(2000),
        };

        let res = execute_request(req).await.unwrap();
        assert_eq!(res.status, 200);
        assert_eq!(res.body, "Hello, Tweak!");
        assert_eq!(res.size_bytes, 13);
        assert!(!res.is_binary);
        assert!(res.headers.iter().any(|h| h.name == "content-type" && h.value == "text/plain"));
    }

    #[tokio::test]
    async fn test_duplicate_headers_preserved() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await.unwrap();
            let response = "HTTP/1.1 200 OK\r\nSet-Cookie: session=abc; Path=/\r\nSet-Cookie: theme=dark; Path=/\r\nContent-Length: 2\r\n\r\nOK";
            socket.write_all(response.as_bytes()).await.unwrap();
            socket.flush().await.unwrap();
        });

        let req = HttpRequest {
            method: "GET".to_string(),
            url: format!("http://127.0.0.1:{port}/cookies"),
            headers: None,
            body: None,
            timeout_ms: Some(2000),
        };

        let res = execute_request(req).await.unwrap();
        let cookies: Vec<&str> = res
            .headers
            .iter()
            .filter(|h| h.name == "set-cookie")
            .map(|h| h.value.as_str())
            .collect();

        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies[0], "session=abc; Path=/");
        assert_eq!(cookies[1], "theme=dark; Path=/");
    }

    #[tokio::test]
    async fn test_binary_response_handling() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await.unwrap();
            // Invalid UTF-8 bytes (binary image or gzip payload)
            let binary_payload = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0xFE];
            let header = format!("HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\n\r\n", binary_payload.len());
            socket.write_all(header.as_bytes()).await.unwrap();
            socket.write_all(&binary_payload).await.unwrap();
            socket.flush().await.unwrap();
        });

        let req = HttpRequest {
            method: "GET".to_string(),
            url: format!("http://127.0.0.1:{port}/image.png"),
            headers: None,
            body: None,
            timeout_ms: Some(2000),
        };

        let res = execute_request(req).await.unwrap();
        assert_eq!(res.status, 200);
        assert!(res.is_binary);
        assert_eq!(res.size_bytes, 10);
        // Verify base64 representation of [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0xFE]
        assert_eq!(res.body, encode_base64(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0xFE]));
    }

    #[tokio::test]
    async fn test_successful_post_with_headers_and_body() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 2048];
            let n = socket.read(&mut buf).await.unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            assert!(request_str.starts_with("POST /submit HTTP/1.1"));
            assert!(request_str.contains("x-test-header: custom-value"));
            assert!(request_str.contains("payload data"));

            let response = "HTTP/1.1 201 Created\r\nContent-Length: 8\r\n\r\nACCEPTED";
            socket.write_all(response.as_bytes()).await.unwrap();
            socket.flush().await.unwrap();
        });

        let mut headers = HashMap::new();
        headers.insert("X-Test-Header".to_string(), "custom-value".to_string());

        let req = HttpRequest {
            method: "POST".to_string(),
            url: format!("http://127.0.0.1:{port}/submit"),
            headers: Some(headers),
            body: Some("payload data".to_string()),
            timeout_ms: Some(2000),
        };

        let res = execute_request(req).await.unwrap();
        assert_eq!(res.status, 201);
        assert_eq!(res.body, "ACCEPTED");
        assert_eq!(res.size_bytes, 8);
        assert!(!res.is_binary);
    }

    // -----------------------------------------------------------------------
    // encode_base64 tests (RFC 4648 Section 10 vectors + coverage tests)
    // Expected values computed with Python's standard library, NOT from the
    // function under test:
    //   python3 -c "import base64; print(base64.b64encode(b'foobar').decode())"
    // -----------------------------------------------------------------------

    #[test]
    fn test_base64_empty() {
        assert_eq!(encode_base64(b""), "");
    }

    #[test]
    fn test_base64_f() {
        assert_eq!(encode_base64(b"f"), "Zg==");
    }

    #[test]
    fn test_base64_fo() {
        assert_eq!(encode_base64(b"fo"), "Zm8=");
    }

    #[test]
    fn test_base64_foo() {
        assert_eq!(encode_base64(b"foo"), "Zm9v");
    }

    #[test]
    fn test_base64_foob() {
        assert_eq!(encode_base64(b"foob"), "Zm9vYg==");
    }

    #[test]
    fn test_base64_fooba() {
        assert_eq!(encode_base64(b"fooba"), "Zm9vYmE=");
    }

    #[test]
    fn test_base64_foobar() {
        assert_eq!(encode_base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_base64_all_256_bytes() {
        // Ground truth from:
        //   python3 -c "import base64; print(base64.b64encode(bytes(range(256))).decode())"
        const EXPECTED: &str = concat!(
            "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDEy",
            "MzQ1Njc4OTo7PD0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWltcXV5fYGFiY2Rl",
            "ZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn+AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeY",
            "mZqbnJ2en6ChoqOkpaanqKmqq6ytrq+wsbKztLW2t7i5uru8vb6/wMHCw8TFxsfIycrL",
            "zM3Oz9DR0tPU1dbX2Nna29zd3t/g4eLj5OXm5+jp6uvs7e7v8PHy8/T19vf4+fr7/P3+/w=="
        );
        let all_bytes: Vec<u8> = (0u8..=255).collect();
        assert_eq!(encode_base64(&all_bytes), EXPECTED);
    }

    #[test]
    fn test_base64_one_mib_length() {
        // Output length must be exactly 4 * ceil(n / 3) for any n-byte input.
        let n = 1024 * 1024usize;
        let buf = vec![0xABu8; n];
        let encoded = encode_base64(&buf);
        let expected_len = n.div_ceil(3) * 4; // 4 * ceil(n/3)
        assert_eq!(encoded.len(), expected_len, "1 MiB: output length mismatch");
    }

    // -----------------------------------------------------------------------
    // normalize_url unit tests
    // -----------------------------------------------------------------------

    // -- Explicit scheme passthrough --

    #[test]
    fn test_normalize_url_explicit_https_passthrough() {
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
    }

    #[test]
    fn test_normalize_url_explicit_http_passthrough() {
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
    }

    #[test]
    fn test_normalize_url_uppercase_scheme_passthrough() {
        // Already has "://" -- returned as-is (trimmed).
        // Url::parse normalises the scheme to lowercase later.
        assert_eq!(normalize_url("HTTP://EXAMPLE.COM"), "HTTP://EXAMPLE.COM");
    }

    #[test]
    fn test_normalize_url_trims_whitespace() {
        assert_eq!(normalize_url("  https://example.com  "), "https://example.com");
    }

    #[test]
    fn test_normalize_url_spaces_in_path_no_panic() {
        let result = normalize_url("https://example.com/a b");
        assert_eq!(result, "https://example.com/a b");
    }

    // -- Local addresses -> http:// --

    #[test]
    fn test_normalize_url_localhost_gets_http() {
        assert_eq!(normalize_url("localhost"), "http://localhost");
    }

    #[test]
    fn test_normalize_url_localhost_port_gets_http() {
        // normalize_url does not add a trailing slash; that is done by Url::parse.
        assert_eq!(normalize_url("localhost:3000"), "http://localhost:3000");
    }

    #[test]
    fn test_normalize_url_localhost_port_with_path_gets_http() {
        assert_eq!(normalize_url("localhost:3000/api"), "http://localhost:3000/api");
    }

    #[test]
    fn test_normalize_url_localhost_subdomain_gets_http() {
        assert_eq!(normalize_url("app.localhost:4000"), "http://app.localhost:4000");
    }

    #[test]
    fn test_normalize_url_127_loopback_gets_http() {
        assert_eq!(normalize_url("127.0.0.1:8080/x"), "http://127.0.0.1:8080/x");
    }

    #[test]
    fn test_normalize_url_127_other_loopback_gets_http() {
        assert_eq!(normalize_url("127.100.200.1/api"), "http://127.100.200.1/api");
    }

    #[test]
    fn test_normalize_url_ipv6_loopback_gets_http() {
        assert_eq!(normalize_url("[::1]:8080/x"), "http://[::1]:8080/x");
    }

    #[test]
    fn test_normalize_url_private_10_gets_http() {
        assert_eq!(normalize_url("10.0.0.1/api"), "http://10.0.0.1/api");
    }

    #[test]
    fn test_normalize_url_private_172_16_gets_http() {
        assert_eq!(normalize_url("172.16.0.1/api"), "http://172.16.0.1/api");
    }

    #[test]
    fn test_normalize_url_private_172_31_gets_http() {
        assert_eq!(normalize_url("172.31.255.255/api"), "http://172.31.255.255/api");
    }

    #[test]
    fn test_normalize_url_private_192_168_gets_http() {
        assert_eq!(normalize_url("192.168.1.100/api"), "http://192.168.1.100/api");
    }

    // -- Public addresses -> https:// --

    #[test]
    fn test_normalize_url_public_domain_gets_https() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
    }

    #[test]
    fn test_normalize_url_public_domain_with_path_gets_https() {
        assert_eq!(normalize_url("httpbin.org/get"), "https://httpbin.org/get");
    }

    #[test]
    fn test_normalize_url_public_ip_gets_https() {
        // 8.8.8.8 is not a private range
        assert_eq!(normalize_url("8.8.8.8/dns"), "https://8.8.8.8/dns");
    }

    #[test]
    fn test_normalize_url_172_15_not_private_gets_https() {
        // 172.15.x is outside the RFC 1918 range (172.16-31)
        assert_eq!(normalize_url("172.15.0.1/api"), "https://172.15.0.1/api");
    }

    #[test]
    fn test_normalize_url_172_32_not_private_gets_https() {
        assert_eq!(normalize_url("172.32.0.1/api"), "https://172.32.0.1/api");
    }

    // -- Single-label hostnames -> http:// --

    #[test]
    fn test_normalize_url_single_label_gets_http() {
        assert_eq!(normalize_url("myserver:8080"), "http://myserver:8080");
    }

    #[test]
    fn test_normalize_url_single_label_no_port_gets_http() {
        assert_eq!(normalize_url("devbox"), "http://devbox");
    }

    #[test]
    fn test_normalize_url_single_label_with_path_gets_http() {
        assert_eq!(normalize_url("printer/status"), "http://printer/status");
    }

    // -- mDNS / local-network TLD hostnames -> http:// --

    #[test]
    fn test_normalize_url_dot_local_gets_http() {
        assert_eq!(normalize_url("mypi.local:8080/api"), "http://mypi.local:8080/api");
    }

    #[test]
    fn test_normalize_url_dot_internal_gets_http() {
        assert_eq!(normalize_url("db.internal/health"), "http://db.internal/health");
    }

    #[test]
    fn test_normalize_url_dot_lan_gets_http() {
        assert_eq!(normalize_url("router.lan"), "http://router.lan");
    }

    // -- CGNAT / shared address 100.64.0.0/10 -> http:// --

    #[test]
    fn test_normalize_url_cgnat_100_64_gets_http() {
        assert_eq!(normalize_url("100.64.0.1/api"), "http://100.64.0.1/api");
    }

    #[test]
    fn test_normalize_url_cgnat_100_127_gets_http() {
        assert_eq!(normalize_url("100.127.255.255/api"), "http://100.127.255.255/api");
    }

    #[test]
    fn test_normalize_url_cgnat_boundary_100_128_gets_https() {
        // 100.128.x is outside the /10 range -- must be treated as public
        assert_eq!(normalize_url("100.128.0.1/api"), "https://100.128.0.1/api");
    }

    // -- Link-local 169.254.0.0/16 -> http:// --

    #[test]
    fn test_normalize_url_link_local_gets_http() {
        assert_eq!(normalize_url("169.254.1.1/api"), "http://169.254.1.1/api");
    }

    // -- Public hostname that must still get https:// (regression guard) --

    #[test]
    fn test_normalize_url_public_api_host_gets_https() {
        // "api.internal.example.com" ends with ".com", not ".internal"
        assert_eq!(
            normalize_url("api.internal.example.com/v1"),
            "https://api.internal.example.com/v1"
        );
    }

    // -- Verify ftp:// error message mentions http and https (task 1) --
    // (The test already existed as test_execute_request_ftp_scheme_rejected;
    //  this alias makes the verification intent explicit in the test list.)

    // -- Verify [::1]:8080/x -> http://[::1]:8080/x (task 1) --
    // (Already covered by test_normalize_url_ipv6_loopback_gets_http.)

    #[tokio::test]
    async fn test_execute_request_empty_url_rejected() {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: "".to_string(),
            headers: None,
            body: None,
            timeout_ms: Some(1000),
        };
        let err = execute_request(req).await.unwrap_err();
        match err {
            HttpError::InvalidUrl(msg) => {
                assert!(!msg.is_empty(), "Error message should not be empty");
            }
            other => panic!("Expected InvalidUrl, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_execute_request_ftp_scheme_rejected() {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: "ftp://x".to_string(),
            headers: None,
            body: None,
            timeout_ms: Some(1000),
        };
        let err = execute_request(req).await.unwrap_err();
        match err {
            HttpError::InvalidUrl(msg) => {
                assert!(
                    msg.contains("http") && msg.contains("https"),
                    "Error should mention http and https, got: {msg}"
                );
            }
            other => panic!("Expected InvalidUrl, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_empty_url() {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: "   ".to_string(),
            headers: None,
            body: None,
            timeout_ms: Some(1000),
        };

        let err = execute_request(req).await.unwrap_err();
        match err {
            HttpError::InvalidUrl(_) => {}
            other => panic!("Expected InvalidUrl, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_network_failure() {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: "http://127.0.0.1:1/nonexistent".to_string(),
            headers: None,
            body: None,
            timeout_ms: Some(1000),
        };

        let err = execute_request(req).await.unwrap_err();
        match err {
            HttpError::NetworkFailure(_) => {}
            other => panic!("Expected NetworkFailure, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (_socket, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_millis(600)).await;
        });

        let req = HttpRequest {
            method: "GET".to_string(),
            url: format!("http://127.0.0.1:{port}/slow"),
            headers: None,
            body: None,
            timeout_ms: Some(100),
        };

        let err = execute_request(req).await.unwrap_err();
        match err {
            HttpError::Timeout(_) => {}
            other => panic!("Expected Timeout, got {other:?}"),
        }
    }
}
