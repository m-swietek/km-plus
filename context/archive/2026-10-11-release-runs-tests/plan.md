# Release runs tests — Implementation Plan

## Overview

Workflow wydania (`.github/workflows/release.yml`) buduje i podpisuje instalator, ale nie uruchamia żadnych testów Rusta — 24 testy sejfu z S-01 działają dziś tylko lokalnie jako bramki faz. Dokładamy jeden krok `cargo test --locked` przed `tauri-apps/tauri-action`, tak żeby czerwony test blokował powstanie draftu wydania, i dowodzimy, że bramka naprawdę blokuje, czerwonym przebiegiem na gałęzi tymczasowej — bez dotykania opublikowanego wydania 0.1.3.

Źródło: ustalenie z S-01 (`context/archive/2026-10-09-password-unlocked-vault/`), podjęte 2026-10-09 jako osobna mała zmiana.

## Current State Analysis

- `.github/workflows/release.yml:13-43` — kroki: checkout → setup-node (`cache: npm`) → `dtolnay/rust-toolchain@stable` → `swatinem/rust-cache@v2` (`workspaces: './src-tauri -> target'`) → `npm ci` → `tauri-apps/tauri-action@v1` (draft, tag `app-v__VERSION__`). Brak kroku testów.
- Jedyną pośrednią bramką jest `beforeBuildCommand: npm run build` (`src-tauri/tauri.conf.json:7-9`), czyli `tsc` strict frontendu; testy Rusta (`#[cfg(test)]`) nie są w ogóle kompilowane w buildzie release.
- `src-tauri/Cargo.lock` jest w repo, więc `--locked` działa i wymusza zgodność z lockiem.
- Testy: 21 w `src-tauri/src/vault/tests.rs`, 3 w `src-tauri/src/vault/commands.rs`; wykonują się < 1 s, koszt to kompilacja profilu dev (pokryta przez istniejący `rust-cache`). `profile.dev.package.argon2/blake2` mają `opt-level = 3` (`src-tauri/Cargo.toml`).
- Wydania: `app-v0.1.0`…`app-v0.1.3`; 0.1.3 jest opublikowane jako Latest, a `tauri.conf.json` ma wersję 0.1.3 — uruchomienie workflow z `main` bez podbicia wersji uderzyłoby w opublikowane wydanie.
- Workflow ma wyłącznie trigger `workflow_dispatch`, który można uruchomić z dowolnej gałęzi (`--ref`), o ile plik istnieje na gałęzi domyślnej (istnieje).

## Desired End State

- Każde uruchomienie `release.yml` najpierw wykonuje `cargo test --locked` w `src-tauri/` na runnerze Windows; czerwony test kończy job przed `npm ci` i przed `tauri-action`, więc nie powstaje żaden tag ani draft.
- Istnieje zapisany dowód: run na gałęzi tymczasowej z celowo zepsutym kodem zakończony porażką w kroku „Rust tests”, z pominiętym `tauri-action` i bez nowego tagu/wydania.
- Gałąź tymczasowa usunięta lokalnie i zdalnie; `main` z nowym krokiem wypchnięty.

### Key Discoveries:

- `.github/workflows/release.yml:23-26` — istniejący `rust-cache` z `workspaces: './src-tauri -> target'`; krok testów po nim korzysta z cache bez dodatkowej konfiguracji.
- `src-tauri/src/vault/mod.rs` — `MIN_PASSWORD_CHARS = 8`; zmiana na 7 powoduje porażkę `password_shorter_than_8_chars_rejected` (wygodny, realny „break” w kodzie produkcyjnym).
- `context/deployment/deploy-plan.md:98` — publikacja i usuwanie wydań/tagów należą wyłącznie do człowieka.
- `src-tauri/tauri.conf.json:4` — wersja 0.1.3 = tag opublikowanego wydania; stąd zabezpieczenie wersją `0.0.0` na gałęzi tymczasowej.

## What We're NOT Doing

- `cargo audit` ani `clippy` w pipeline (decyzja planu; audyt zostaje lokalny).
- Osobny workflow CI na push/PR — autor jest jeden, wydania ręczne.
- Podbijanie wersji i wydanie testowe 0.1.4 dla ścieżki zielonej; zielony przebieg zobaczymy przy następnym realnym wydaniu (S-02).
- Testy frontendu (brak runnera) i zmiany w `tauri-action`/sekretach.
- Aktualizacja nieaktualnego opisu stanu w `AGENTS.md:5` („untouched scaffold … no tests … CI”) — osobna sprawa.

## Implementation Approach

Krok testów trafia przed `npm ci`, bo nie potrzebuje Node i ma przerwać job jak najwcześniej. Dowód działania robimy na dokładnie tym pliku workflow, który wyląduje na `main`: commit z krokiem zostaje lokalnie na `main`, gałąź tymczasowa odgałęzia się od niego i dodaje tylko celowe zepsucie oraz zabezpieczenie wersji. `main` wypychamy dopiero po czerwonym przebiegu i usunięciu gałęzi.

## Critical Implementation Details

- **Zabezpieczenie przed uderzeniem w opublikowane wydanie (State sequencing).** Na gałęzi tymczasowej wersja w `src-tauri/tauri.conf.json` zmieniona na `0.0.0` w tym samym commicie co zepsucie testu. Gdyby bramka nie zadziałała, `tauri-action` utworzy co najwyżej draft `app-v0.0.0` (niewidoczny dla updatera, niższa wersja), a nie zmodyfikuje `app-v0.1.3`. Przed wysłaniem dispatchu sprawdź, że na wypchniętej gałęzi `tauri.conf.json` ma `0.0.0`. Jeśli run mimo wszystko przejdzie krok testów — natychmiast `gh run cancel`, a ewentualny draft `app-v0.0.0` i tag usuwa użytkownik.
- **Kolejność pushy.** Gałąź tymczasowa wypychana przed `main`; `main` z krokiem dopiero po udowodnieniu porażki. Commit „break” nigdy nie trafia na `main`.

## Phase 1: Krok testów w workflow

### Overview

Dodanie kroku `cargo test --locked` do `release.yml` i krótkiej notatki w planie wdrożenia; weryfikacja lokalna. Commit zostaje lokalnie (bez pusha) do Fazy 2.

### Changes Required:

#### 1. Workflow wydania

**File**: `.github/workflows/release.yml`

**Intent**: Zablokować powstanie draftu, gdy którykolwiek test Rusta nie przechodzi.

**Contract**: nowy krok między `Rust cache` a `install frontend dependencies`: `name: Rust tests`, `run: cargo test --locked`, `working-directory: src-tauri`, z jednolinijkowym komentarzem w stylu pliku (czerwony test = brak draftu). Pozostałe kroki bez zmian.

#### 2. Notatka w planie wdrożenia

**File**: `context/deployment/deploy-plan.md`

**Intent**: Zostawić ślad, że pipeline ma bramkę testów i od kiedy.

**Contract**: jeden datowany wpis w sekcji `## Przebieg wykonania` (2026-10-11: `release.yml` uruchamia `cargo test --locked` przed buildem; czerwony test = brak draftu; zmiana `release-runs-tests`).

### Success Criteria:

#### Automated Verification:

- Polecenie kroku przechodzi lokalnie: `cargo test --locked` w `src-tauri/`
- Krok stoi przed `npm ci` i `tauri-action`: numer linii `name: Rust tests` w `release.yml` jest mniejszy niż numery linii `npm ci` i `tauri-apps/tauri-action`

#### Manual Verification:

- Przegląd diffu `release.yml`: dodany wyłącznie krok „Rust tests”, wcięcia zgodne z sąsiednimi krokami

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase. Commit lands locally only; push happens at the end of Phase 2.

---

## Phase 2: Dowód bramki czerwonym przebiegiem

### Overview

Gałąź tymczasowa z celowo zepsutym kodem i wersją `0.0.0`, uruchomienie workflow z tej gałęzi, potwierdzenie porażki w kroku testów bez skutków dla wydań, sprzątnięcie gałęzi i wypchnięcie `main`. Push gałęzi, dispatch i push `main` wymagają wyraźnej zgody użytkownika.

### Changes Required:

#### 1. Gałąź tymczasowa (nie trafia na `main`)

**File**: gałąź `release-gate-redcheck` od lokalnego `main` z Fazy 1: `src-tauri/src/vault/mod.rs`, `src-tauri/tauri.conf.json`

**Intent**: Realne zepsucie zachowania chronionego testem i zabezpieczenie wersji.

**Contract**: jeden commit: `MIN_PASSWORD_CHARS` 8 → 7 oraz `version` w `tauri.conf.json` → `0.0.0`; push tylko tej gałęzi; `gh workflow run release.yml --ref release-gate-redcheck`.

#### 2. Sprzątanie i publikacja kroku

**File**: brak zmian w plikach; operacje git

**Intent**: Usunąć gałąź tymczasową (zdalnie i lokalnie), wrócić na `main`, wypchnąć `main`; zapisać numer runu jako dowód w `change.md` (`## Notes`).

**Contract**: `git push origin --delete release-gate-redcheck`, `git branch -D release-gate-redcheck`, `git push origin main`; wpis w `context/changes/release-runs-tests/change.md`.

### Success Criteria:

#### Automated Verification:

- Run z gałęzi tymczasowej kończy się porażką w kroku „Rust tests”, a krok `tauri-action` jest pominięty: `gh run view <id> --json jobs`
- Żaden nowy tag ani wydanie nie powstały: `gh release list` i `git ls-remote --tags origin` pokazują tylko `app-v0.1.0`…`app-v0.1.3`
- Gałąź tymczasowa usunięta: `git ls-remote --heads origin release-gate-redcheck` nic nie zwraca
- `main` z krokiem wypchnięty: `git status -sb` pokazuje `main...origin/main` bez rozbieżności

#### Manual Verification:

- Log kroku „Rust tests” w runie pokazuje padający `password_shorter_than_8_chars_rejected` i pozostałe testy uruchomione (nie „0 tests”)

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Testing Strategy

### Unit Tests:

- Bez nowych testów; krok uruchamia istniejące 24 testy Rusta.

### Integration Tests:

- Czerwony przebieg workflow na gałęzi tymczasowej (Faza 2) jako test samej bramki.

### Manual Testing Steps:

1. Przejrzeć diff `release.yml`.
2. Otworzyć log runu z gałęzi tymczasowej i potwierdzić padający test oraz pominięty `tauri-action`.

## Performance Considerations

- Krok dodaje kompilację profilu dev + testów (~1–2 min przy ciepłym cache, dłużej przy pierwszym runie po zmianie zależności). Akceptowalne przy ręcznych, rzadkich wydaniach.

## Migration Notes

- Nie dotyczy (brak danych; zmiana tylko pipeline'u).

## References

- Workflow: `.github/workflows/release.yml:13-43`
- Plan wdrożenia i bramki ręczne: `context/deployment/deploy-plan.md` (kroki 9–13, `## Granice`)
- Testy chronione bramką: `src-tauri/src/vault/tests.rs`, `src-tauri/src/vault/commands.rs`
- Pochodzenie decyzji: `context/archive/2026-10-09-password-unlocked-vault/`

## Progress

> Convention: `- [ ]` pending, `- [x]` done. Append ` — <commit sha>` when a step lands. Do not rename step titles. See `references/progress-format.md`.

### Phase 1: Krok testów w workflow

#### Automated

- [x] 1.1 Polecenie kroku przechodzi lokalnie: `cargo test --locked` w `src-tauri/` — 45e6ae2
- [x] 1.2 Krok stoi przed `npm ci` i `tauri-action`: numer linii `name: Rust tests` w `release.yml` jest mniejszy niż numery linii `npm ci` i `tauri-apps/tauri-action` — 45e6ae2

#### Manual

- [x] 1.3 Przegląd diffu `release.yml`: dodany wyłącznie krok „Rust tests”, wcięcia zgodne z sąsiednimi krokami — 45e6ae2

### Phase 2: Dowód bramki czerwonym przebiegiem

#### Automated

- [x] 2.1 Run z gałęzi tymczasowej kończy się porażką w kroku „Rust tests”, a krok `tauri-action` jest pominięty: `gh run view <id> --json jobs` — b23621d
- [x] 2.2 Żaden nowy tag ani wydanie nie powstały: `gh release list` i `git ls-remote --tags origin` pokazują tylko `app-v0.1.0`…`app-v0.1.3` — b23621d
- [x] 2.3 Gałąź tymczasowa usunięta: `git ls-remote --heads origin release-gate-redcheck` nic nie zwraca — b23621d
- [x] 2.4 `main` z krokiem wypchnięty: `git status -sb` pokazuje `main...origin/main` bez rozbieżności — b23621d

#### Manual

- [x] 2.5 Log kroku „Rust tests” w runie pokazuje padający `password_shorter_than_8_chars_rejected` i pozostałe testy uruchomione (nie „0 tests”) — b23621d
