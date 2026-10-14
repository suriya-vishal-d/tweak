# AGENTS.md — Tweak Project Conventions

> **Tweak** is a lightweight, offline API client for Linux built with Tauri v2 + React + TypeScript.

---

## Project Layout

```
tweak/
├── AGENTS.md               ← this file
├── docs/
│   └── roadmap.md          ← milestone roadmap
├── schemas/                ← JSON schemas for collections, requests, environments
├── src/
│   ├── components/         ← React UI components (each in its own subfolder)
│   ├── store/              ← Zustand stores and context providers
│   ├── lib/                ← pure utility/helper functions (no React)
│   ├── types/              ← shared TypeScript types and interfaces
│   └── styles/             ← global CSS, theme variables, resets
├── src-tauri/
│   ├── src/                ← Rust source (Tauri commands, state)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html
├── vite.config.ts
├── tsconfig.json
├── eslint.config.js
├── .prettierrc
└── vitest.config.ts
```

---

## Tech Stack

| Layer         | Choice                             |
| ------------- | ---------------------------------- |
| Desktop shell | Tauri v2                           |
| Frontend      | React 18 + TypeScript              |
| Build tool    | Vite 5                             |
| State         | Zustand                            |
| Styling       | Plain CSS variables (no framework) |
| Lint          | ESLint (flat config) + Prettier    |
| Tests         | Vitest + React Testing Library     |
| Backend/IPC   | Rust (Tauri commands)              |

---

## Principles

- Keep it simple. Build normal API client features first; do not add advanced features before the milestone calls for them.

---

## Coding Conventions

### TypeScript / React

- Strict TypeScript (`"strict": true` in tsconfig).
- Functional components only; no class components.
- Props interfaces named `<ComponentName>Props`.
- One component per file; filename matches component name (PascalCase).
- Barrel `index.ts` files are allowed but not required.
- No `any` types. Use `unknown` when the type is truly unknown.
- No `console.log` in production code; use a `lib/logger.ts` wrapper.

### CSS / Theming

- All colours, spacing, and radii go through CSS custom properties defined in `src/styles/theme.css`.
- Never hard-code colour values in component stylesheets.
- Theme classes applied on `<html>`: `.theme-light`, `.theme-dark`.
- System theme is read via `prefers-color-scheme` media query.

### Rust

- Follow `cargo clippy --all-targets -- -D warnings` — zero warnings allowed.
- All Tauri commands are in `src-tauri/src/commands/` (one module per feature).
- No `unwrap()` in production code; use proper `Result` propagation.
- Keep `main.rs` minimal — only app setup.

### Git

- Commits follow Conventional Commits: `feat:`, `fix:`, `chore:`, `docs:`, etc.
- One logical change per commit.

---

## Milestones

See [docs/roadmap.md](docs/roadmap.md) for the full milestone breakdown.

---

## No-go List

The following are explicitly forbidden in this codebase:

- No login / authentication UI
- No analytics or telemetry
- No outbound network calls from the frontend (only via Tauri IPC + Rust)
- No third-party CSS frameworks (Tailwind, Bootstrap, etc.)
- No `any` TypeScript type escapes
