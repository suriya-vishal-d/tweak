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

## Milestone M1 — Request Builder (Core UI)

**Goal:** Build the core request-sending UI: URL bar, method selector, headers/body tabs, response panel.

### Tasks
- [ ] `RequestBar` component — URL input + HTTP method dropdown (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- [ ] `HeadersEditor` component — key/value pair list with add/remove rows
- [ ] `BodyEditor` component — raw text area with content-type selector (JSON, plain, form-data)
- [ ] `ResponsePanel` component — status code badge, response time, response body (syntax-highlighted JSON)
- [ ] Tauri command `send_request` in Rust using `reqwest` (offline, no auth)
- [ ] Zustand store `useRequestStore` — holds current request state
- [ ] Vitest unit tests for store and utility functions

### Deliverables
- User can type a URL, pick a method, add headers/body, and click Send
- Response displayed in the panel (status, time, body)
- All lint + tests green

---

## Milestone M2 — Collections & History

**Goal:** Persist requests as named collections and maintain a request history.

### Tasks
- [ ] JSON schema for collections (`schemas/collection.schema.json`)
- [ ] Tauri commands: `save_collection`, `load_collections`, `delete_collection`
- [ ] `CollectionSidebar` component — tree view of saved requests
- [ ] `HistoryPanel` component — last N requests with timestamps
- [ ] File-based storage via Tauri's app data directory (no remote DB)
- [ ] Import/Export collection as JSON file

---

## Milestone M3 — Environments & Variables

**Goal:** Support named environments with variable substitution in requests.

### Tasks
- [ ] JSON schema for environments (`schemas/environment.schema.json`)
- [ ] `EnvironmentEditor` component — manage key/value env variables
- [ ] Variable substitution in URL and headers (`{{variable}}` syntax)
- [ ] Tauri commands: `save_environment`, `load_environments`

---

## Milestone M4 — Polish & Release Prep

**Goal:** Make it shippable as an AUR package / AppImage.

### Tasks
- [ ] App icon (SVG → PNG sizes via script)
- [ ] `.desktop` file for Linux application menu
- [ ] PKGBUILD for AUR
- [ ] AppImage build via `tauri build`
- [ ] README with install instructions
- [ ] Keyboard shortcuts (Ctrl+Enter to send, Ctrl+S to save, etc.)
- [ ] Accessibility audit (ARIA labels, keyboard navigation)

---

## Non-Goals (Forever)

- No cloud sync
- No login / OAuth
- No analytics or telemetry
- No Windows / macOS builds (focus: Arch Linux)

