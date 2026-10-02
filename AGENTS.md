# kmPlus (`km-plus`) — Repository Guidelines

Single-user, offline desktop app for one LPG-converted car: fuel-up log with consumption computed separately for LPG and petrol, plus a service register with mileage/date reminders. Tauri 2 (Rust backend) + React 19 + TypeScript + Vite.

**Current state: untouched `create-tauri-app` react-ts scaffold.** The only code is the template's `greet` command and demo UI — no domain code, persistence, auth, tests, linter, or CI exist yet. Requirements live in `context/foundation/prd.md` (written in Polish; FR-001…FR-011, US-01…US-03) and stack rationale in `context/foundation/tech-stack.md`. Read the PRD before designing any feature.

## Hard product constraints (from the PRD — binary, no exceptions)

- **No network egress of user data, ever.** No telemetry, cloud sync, update pings carrying data, or external APIs — not even behind user consent. All MVP features must work fully offline.
- **Data at rest is encrypted with a key derived from the user's password** (FR-001). The password gates decryption, not just the UI. The scaffold does not include this layer; it has to be added in the Rust backend.
- **No password recovery path** — no security question, recovery key, or plaintext/emergency export. This is a decided product property, not a gap; do not add one.
- **Data loss is a critical failure.** The app is the sole source of history; treat migrations and destructive writes accordingly.
- **A wrong consumption figure is worse than no figure.** When inputs are insufficient, show nothing rather than an estimate.
- Out of scope by decision: multiple vehicles, multiple users/roles, mobile companion, background jobs, notifications outside the app, built-in service-schedule database, WCAG compliance.

## Domain rules that are easy to get wrong

- Consumption for a fuel is computed against the previous fill-up **of the same fuel**, not the chronologically adjacent entry.
- Only **full-tank** fill-ups close a consumption period; partial top-ups never produce a consumption result.
- The one-time starting point (date + odometer, FR-002) makes the very first fill-up already yield a result.
- Editing or deleting a fill-up must recompute consumption.
- Accepted simplification: kilometres driven on petrol during start-up count toward LPG distance (LPG figure is slightly inflated, systematically). Do not "fix" this.
- A reminder fires when the **first** of the configured thresholds (mileage or date) is reached, using the latest known odometer (from a fill-up or a standalone odometer entry) or today's date. It is evaluated on app open and shown on every open until marked done.
- Marking a recurring service done creates the next occurrence with thresholds counted from the **current** odometer and date, not from the originally planned threshold.

## Commands

Run from the repo root unless noted. npm is the JS package manager; cargo builds the backend.

- `cargo check` / `cargo test` / `cargo test <name>` — run inside `src-tauri/`.
- `cargo audit` (inside `src-tauri/`) and `npm audit` — dependency audits used at bootstrap.

There is no test runner for the frontend and no lint/format script yet; `tsc` with `strict`, `noUnusedLocals`, and `noUnusedParameters` is the only static gate, so unused imports/params fail `npm run build`.

## Architecture

- `src/` — React frontend, rendered in the Tauri webview. Talks to the backend only through `invoke("<command>", args)` from `@tauri-apps/api/core`.
- `src-tauri/src/lib.rs` — the backend entry point. `#[tauri::command]` functions must be registered in `tauri::generate_handler![...]` inside `run()` or the frontend call fails at runtime. `main.rs` is only a thin wrapper calling `km_plus_lib::run()`.
- `src-tauri/capabilities/default.json` — permission allowlist for the `main` window. Adding a Tauri plugin requires three changes: the crate in `Cargo.toml`, `.plugin(...)` in `run()`, and the plugin's permission here.
- `src-tauri/tauri.conf.json` — dev URL is pinned to `http://localhost:1420` and `vite.config.ts` uses `strictPort`, so the two must change together. `app.security.csp` is currently `null`.
- The Rust lib crate is named `km_plus_lib` (the `_lib` suffix avoids a Windows bin/lib name clash); keep `main.rs` in sync if it is ever renamed.

## `context/` workflow docs

- `context/foundation/` — living cross-change docs (PRD, tech stack, shape notes); edit in place. `context/foundation/archive/` holds superseded versions and is not read routinely.
- `context/changes/<change-id>/` — artifacts scoped to one in-flight change. `bootstrap-verification/verification.md` records how the scaffold was produced and audited.
- `context/archive/` — completed changes; **never write here**.
