# Release runs tests — Plan Brief

> Full plan: `context/changes/release-runs-tests/plan.md`

## What & Why

Workflow wydania buduje i podpisuje instalator, ale nie uruchamia testów Rusta — 24 testy sejfu z S-01 działają tylko lokalnie. Dokładamy krok `cargo test --locked` przed `tauri-action`, żeby czerwony test blokował powstanie draftu wydania.

## Starting Point

`.github/workflows/release.yml`: checkout → Node → Rust + cache → `npm ci` → `tauri-action` (draft). Jedyna pośrednia bramka to `tsc` frontendu w `beforeBuildCommand`. Wydanie 0.1.3 jest opublikowane, a `tauri.conf.json` ma wersję 0.1.3.

## Desired End State

Każde wydanie najpierw uruchamia `cargo test --locked` na runnerze Windows; czerwony test kończy job przed buildem, bez tagu i draftu. Istnieje dowód: run z gałęzi tymczasowej z celowo zepsutym kodem padł w kroku testów, a żadne wydanie nie powstało.

## Key Decisions Made

| Decision | Choice | Why (1 sentence) |
| --- | --- | --- |
| Zakres bramki | Tylko `cargo test --locked` | Dokładnie uzgodniony zakres; audyt i clippy zostają lokalnie. |
| Miejsce kroku | Po `Rust cache`, przed `npm ci` | Nie potrzebuje Node, przerywa job najwcześniej, korzysta z istniejącego cache. |
| Weryfikacja | Czerwony run na gałęzi tymczasowej | Dowodzi blokady bez skutków dla wydań; zielony przebieg przy S-02. |
| Zabezpieczenie | Wersja `0.0.0` na gałęzi tymczasowej | Gdyby bramka zawiodła, powstałby tylko fikcyjny draft, nie zmiana opublikowanego 0.1.3. |
| Kolejność pushy | Gałąź tymczasowa, potem `main` | Na `main` ląduje dokładnie ten plik workflow, który został udowodniony. |

## Scope

**In scope:**
- Krok „Rust tests” w `release.yml`
- Wpis w `context/deployment/deploy-plan.md` → `## Przebieg wykonania`
- Czerwony run na gałęzi `release-gate-redcheck`, jej usunięcie, push `main`

**Out of scope:**
- `cargo audit`, `clippy`, osobny workflow CI, testy frontendu
- Wydanie testowe 0.1.4
- Aktualizacja nieaktualnego opisu w `AGENTS.md`

## Architecture / Approach

Commit z krokiem zostaje lokalnie na `main`; gałąź tymczasowa odgałęzia się od niego i dodaje jeden commit (`MIN_PASSWORD_CHARS` 8 → 7, wersja `0.0.0`). Po wypchnięciu tylko gałęzi `gh workflow run release.yml --ref release-gate-redcheck` musi paść w „Rust tests” z pominiętym `tauri-action`. Potem gałąź znika, a `main` jest wypychany.

## Phases at a Glance

| Phase | What it delivers | Key risk |
| --- | --- | --- |
| 1. Krok testów w workflow | Krok w `release.yml` + notatka, commit lokalny | Złe wcięcie YAML wychodzi dopiero na GitHubie (Faza 2) |
| 2. Dowód bramki | Czerwony run, brak nowych wydań, gałąź usunięta, `main` wypchnięty | Bramka nie zadziała → zabezpieczenie `0.0.0` + `gh run cancel` |

**Prerequisites:** `gh` zalogowany; cargo w PATH (`$HOME\.cargo\bin`); zgoda użytkownika na push gałęzi, dispatch i push `main`.
**Estimated effort:** ~1 sesja, 2 fazy (głównie czekanie na run CI).

## Open Risks & Assumptions

- Zielona ścieżka na CI zobaczona dopiero przy następnym realnym wydaniu (S-02).
- Gałąź tymczasowa jest przez kilka minut widoczna w publicznym repo.
- Pierwszy run po zmianie zależności będzie dłuższy (zimny cache profilu dev).

## Success Criteria (Summary)

- Wydanie nie powstanie, jeśli którykolwiek test Rusta jest czerwony.
- Dowód blokady zapisany w notatkach zmiany, bez żadnych skutków dla opublikowanych wydań.
