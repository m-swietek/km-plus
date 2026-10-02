---
bootstrapped_at: 2026-09-29T19:58:17Z
starter_id: tauri
starter_name: Tauri
project_name: km-plus
language_family: rust
package_manager: npm
cwd_strategy: subdir-then-move
bootstrapper_confidence: verified
phase_3_status: ok
audit_command: cargo audit
---

## Hand-off

```yaml
starter_id: tauri
package_manager: cargo
project_name: km-plus
hints:
  language_family: rust
  team_size: solo
  deployment_target: self-host
  ci_provider: github-actions
  ci_default_flow: manual-promotion
  bootstrapper_confidence: verified
  path_taken: standard
  quality_override: false
  self_check_answers: null
  has_auth: true
  has_payments: false
  has_realtime: false
  has_ai: false
  has_background_jobs: false
```

### Why this stack

Jednoosobowy projekt desktopowy budowany po godzinach w 6 tygodni, z twardym wymaganiem pracy w pełni offline i bez wysyłki danych na zewnątrz. Tauri jest wetowanym domyślnym starterem dla pary (desktop, Rust) i przechodzi wszystkie cztery bramki agent-friendly: jawne typy po obu stronach (Rust + TypeScript), silne konwencje układu projektu, obecność w danych treningowych i aktualna dokumentacja. Scaffolding jest sprawdzony end-to-end, więc start projektu nie zje budżetu czasowego. Backend w Ruście jest też właściwym miejscem na wymaganie z FR-001 i wymagań niefunkcjonalnych: lokalna baza szyfrowana kluczem wyprowadzonym z hasła użytkownika — warstwa, której starter nie niesie i którą trzeba dołożyć ręcznie zaraz po zescaffoldowaniu. Dystrybucja to self-host (własny build instalatora), zgodnie z domyślną wartością karty i z modelem jednego użytkownika na jednym komputerze. CI na GitHub Actions z ręczną promocją — wydanie instalatora pozostaje świadomą decyzją, nie efektem ubocznym merge'a. Flagi funkcji: auth włączony; płatności, realtime, AI i zadania w tle są wprost poza zakresem według sekcji Non-Goals.

### Session override

- `package_manager`: `cargo` → `npm` (this run only; `tech-stack.md` unchanged). Reason: `create-tauri-app --manager cargo` supports only Rust-frontend templates (vanilla/Yew/Leptos/Sycamore/Dioxus); the card's `--template react-ts` requires a JS package manager for the frontend. Cargo remains the Rust backend's build tool.

## Pre-scaffold verification

| Signal      | Value                                          | Severity | Notes                                                                 |
| ----------- | ---------------------------------------------- | -------- | --------------------------------------------------------------------- |
| npm package | create-tauri-app v4.7.4 published 2026-09-04   | fresh    | resolved from cmd_template (checked although language_family is rust, since the CLI is npm-distributed) |
| GitHub repo | not run                                        | —        | card `docs_url` is `https://tauri.app` (not GitHub); `gh` CLI also not installed |

Re-checked on 2026-10-02: `create-tauri-app` is still v4.7.4 (modified 2026-09-04) — unchanged, still fresh. GitHub check still not run for the same reasons.

## Scaffold log

**Resolved invocation**: `npm create tauri-app@latest .bootstrap-scaffold -- --template react-ts --manager npm --yes`
**Strategy**: subdir-then-move
**Exit code**: 0
**Files moved**: 38
**Conflicts (.scaffold siblings)**: none
**.gitignore handling**: moved silently (absent in cwd)
**.bootstrap-scaffold cleanup**: deleted

**CLI notes**: create-tauri-app reported missing system dependency: Rust (https://www.rust-lang.org/learn/get-started#installing-rust). Template creation succeeded; dependencies were not installed (`npm install` not run by the template).

**Post-move fix-up**: the CLI derived identifiers from the temp directory name. Renamed to `project_name`:
- `package.json` name: `bootstrap-scaffold` → `km-plus`
- `src-tauri/Cargo.toml` package name: `bootstrap-scaffold` → `km-plus`; lib name `bootstrap_scaffold_lib` → `km_plus_lib`
- `src-tauri/src/main.rs`: `bootstrap_scaffold_lib::run()` → `km_plus_lib::run()`
- `src-tauri/tauri.conf.json`: `productName`, window `title` → `km-plus`; `identifier` → `com.administrator.km-plus`

**Files moved**:

```
.gitignore
.vscode/extensions.json
README.md
index.html
package.json
public/tauri.svg
public/vite.svg
src/App.css
src/App.tsx
src/assets/react.svg
src/main.tsx
src/vite-env.d.ts
src-tauri/.gitignore
src-tauri/Cargo.toml
src-tauri/build.rs
src-tauri/capabilities/default.json
src-tauri/icons/ (18 icon files)
src-tauri/src/lib.rs
src-tauri/src/main.rs
src-tauri/tauri.conf.json
tsconfig.json
tsconfig.node.json
vite.config.ts
```

## Post-scaffold audit

**Tool**: cargo audit
**Summary**: 0 CRITICAL, 0 HIGH, 0 MODERATE, 0 LOW
**Direct vs transitive**: 0/0/0/0 direct of total 0/0/0/0

Re-run on 2026-10-02 (cargo 1.99.0, cargo-audit 0.22.2, run in `src-tauri/`, exit code 0), replacing the original 2026-09-29 record in which the audit failed to run because the Rust toolchain was not installed. Scanned `Cargo.lock`: 459 crate dependencies against 1280 advisories.

#### CRITICAL findings

None.

#### HIGH findings

None.

#### MODERATE findings

None.

#### LOW / INFO findings

No vulnerabilities. Two informational warnings (allowed by cargo-audit, both transitive):

- `proc-macro-error` 1.0.4 — RUSTSEC-2024-0370 — unmaintained. No fix version; resolves when upstream crates drop the dependency.
- `glib` 0.18.5 — RUSTSEC-2024-0429 — unsound: `Iterator` and `DoubleEndedIterator` impls for `glib::VariantStrIter`. Fix version not checked.

Supplementary checks run the same day (not part of the `cargo audit` dispatch):

- `npm audit` on the JS frontend: 0 vulnerabilities across 62 dependencies (6 prod, 57 dev, 38 optional).
- `npx tsc --noEmit`: exit code 0.
- `cargo check` in `src-tauri/`: exit code 0.

## Hints recorded but not acted on

| Hint                    | Value            |
| ----------------------- | ---------------- |
| bootstrapper_confidence | verified         |
| quality_override        | false            |
| path_taken              | standard         |
| self_check_answers      | null             |
| team_size               | solo             |
| deployment_target       | self-host        |
| ci_provider             | github-actions   |
| ci_default_flow         | manual-promotion |
| has_auth                | true             |
| has_payments            | false            |
| has_realtime            | false            |
| has_ai                  | false            |
| has_background_jobs     | false            |

## Next steps

Next: a future skill will set up agent context (CLAUDE.md, AGENTS.md). For now, your project is scaffolded and verified — happy hacking.

Useful manual steps in the meantime:
- Rust toolchain and dependencies are now installed (as of 2026-10-02); run `npm run tauri dev` to start the app.
- `git init` (if you have not already) to start your own repo history — done, initial commit `1e049a8`.
- Review any `.scaffold` siblings the conflict policy created and decide which version of each file to keep (none this run).
- Address audit findings per your project's risk tolerance — the full breakdown is in this log (no vulnerabilities; two informational warnings).
