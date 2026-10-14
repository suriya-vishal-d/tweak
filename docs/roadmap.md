# Tweak — Development Roadmap

> A lightweight, offline-first API client for Linux (Arch-focused) built with Tauri v2 + React + TypeScript.

---

## Milestone M0 — Project Foundation ✅

**Goal:** Scaffold the project with tooling, theming, and folder structure. No functional API features yet.

### Tasks

- [x] Scaffold Tauri v2 + React + TypeScript + Vite (no nesting)
- [x] Add ESLint (flat config), Prettier, Vitest
- [x] `npm run lint` and `npm test` pass
- [x] `cargo clippy --all-targets -- -D warnings` passes
- [x] Light / dark / system theme via CSS variables + toggle button
- [x] Folder layout: `src/components`, `src/store`, `src/lib`, `src/types`, `src/styles`, `docs/`, `schemas/`

### Deliverables

- Working `npm run dev` (Vite dev server)
- Working `npm run tauri dev` (Tauri window)
- Green lint + tests
- Theme toggle visible in UI

---

## Milestone M1 — First Request Through Rust ✅

**Goal:** Send an HTTP request from the frontend through Rust via Tauri IPC using `reqwest` and display the raw response.

### Tasks

- [x] Remove starter `greet` command
- [x] Tauri command `send_http_request` in Rust using `reqwest` with timeout support
- [x] Typed `HttpResponse` (status, headers pairs, body, elapsed_ms, size_bytes, is_binary)
- [x] Typed `HttpError` (invalid URL, network failure, timeout, invalid method, invalid header)
- [x] Request logic in dedicated module (`src-tauri/src/http/`)
- [x] Frontend wrapper in `src/lib/` and minimal UI (method select, URL input, send button, raw response output)
- [x] Rust unit tests with local test server and frontend tests

---

## Milestone M2A — Request Builder

**Goal:** Build the full request builder UI with dedicated parameter, header, body, and authentication tabs.

### Tasks

- [ ] Query params tab (key-value editor synced with URL)
- [ ] Headers tab (key-value editor with enable/disable toggles)
- [ ] Body tab (none, JSON, raw text, form-urlencoded)
- [ ] Auth tab (Bearer token, Basic auth)

---

## Milestone M2B — Response Viewer

**Goal:** Rich response viewer with status formatting, headers, clipboard actions, cancellation, and granular timing.

### Tasks

- [ ] Status badge and response status text
- [ ] Pretty JSON viewer and raw view
- [ ] Response headers tab
- [ ] Copy response body to clipboard
- [ ] Cancel in-flight request (requires Rust-side cancellation)
- [ ] Timing and sizing metrics: total ms, approximate time to first byte / TTFB (time until response headers arrive), payload size

---

## Milestone M3 — Plain-File Persistence

**Goal:** Save and load requests as plain files in a project folder.

### Tasks

- [ ] Save request to plain file in project directory
- [ ] Open and load request from plain file
- [ ] Project folder browser / file sidebar

---

## Milestone M4 — Environments & Variables

**Goal:** Support named environments with dynamic variable replacement.

### Tasks

- [ ] Environment manager with key-value variable editor
- [ ] Variable substitution in URL, headers, params, body, and auth using `{{variable}}` syntax
- [ ] Active environment switcher

---

## Milestone M5 — History, Tabs & cURL

**Goal:** Request history tracking, multi-tab editing, and cURL import/export.

### Tasks

- [ ] Request history log with timestamps
- [ ] Multi-tab request workspace
- [ ] Keyboard shortcuts (Ctrl+Enter to send, Ctrl+S to save)
- [ ] Import from cURL command
- [ ] Export request as cURL command

---

## Milestone M6 — Latency Test

**Goal:** Send the same request N times with a configurable delay to measure latency and reliability.

### Tasks

- [ ] Configure iteration count (N) and delay between requests
- [ ] Sequential execution of the configured request
- [ ] Display min, avg, and max response time
- [ ] Display success and failure counts

---

## Milestone M7 — Packaging & README

**Goal:** Release packaging and documentation.

### Tasks

- [ ] README with installation and usage instructions
- [ ] App icon and `.desktop` file for Linux application menu
- [ ] Release packages (AUR PKGBUILD, AppImage)

### Notes

- **TLS backend:** rustls uses aws-lc-rs, which compiles C code, so CI needs a C compiler (and cmake if the build asks for it).
- **glibc compatibility:** Build release packages on an older Ubuntu LTS image
  (e.g. Ubuntu 22.04 / glibc 2.35) so the resulting binaries run on a wider range
  of distributions without requiring users to have the latest glibc.

---

## Later

The following features are deferred until after the active milestones above:

- Test runner
- Git panel
- OpenAPI import
- CLI
- Diffing
- WebSocket
- Mock server
- Load testing
- Multipart form-data

---

## Non-Goals (Forever)

- No cloud sync
- No login / OAuth
- No analytics or telemetry
- No Windows / macOS builds (focus: Arch Linux)
