# Hasło odblokowuje zaszyfrowane dane (S-01) — Implementation Plan

## Overview

Dokładamy warstwę danych kmPlus: jeden zaszyfrowany plik („sejf”) w katalogu danych aplikacji, odszyfrowywany kluczem wyprowadzonym z hasła użytkownika (Argon2id → XChaCha20-Poly1305). Przy pierwszym uruchomieniu użytkownik ustawia hasło, przy każdym kolejnym odblokowuje nim aplikację. Błędne hasło daje jednoznaczny komunikat i nigdy nie zapisuje niczego na dysku; uszkodzony lub brakujący plik główny nie jest po cichu zastępowany nowym, pustym sejfem, tylko — za zgodą użytkownika — przywracany z ostatniej dobrej kopii. Plik z nowszej wersji aplikacji nie jest otwierany ani nadpisywany. To fundament dla każdego kolejnego kawałka roadmapy (S-02…S-08), które będą zapisywać dane przez ten sejf.

Źródło: `context/foundation/roadmap.md` S-01 (FR-001), PRD §Access Control, §Guardrails, §Non-Functional Requirements; ryzyko migracji z `context/foundation/infrastructure.md:91`.

## Current State Analysis

- Backend to szablon: `src-tauri/src/lib.rs:2-5` — komenda `greet`; `run()` (`lib.rs:8-15`) rejestruje wtyczki updater i opener oraz tylko `greet` w `generate_handler!`.
- `src-tauri/Cargo.toml:20-24` — brak crate'ów do kryptografii i składowania; `edition = "2021"`; profil release ma `panic = "abort"`.
- Frontend: `src/App.tsx` — demo `greet` + logotypy (`App.tsx:50-79`) oraz baner aktualizacji (`App.tsx:13-49`), który musi zostać.
- Tożsamość aplikacji jest zamrożona: `identifier: io.github.m-swietek.kmplus` (`src-tauri/tauri.conf.json:5`), więc katalog danych to `%APPDATA%\io.github.m-swietek.kmplus` (`context/deployment/deploy-plan.md:90`).
- Uprawnienia: `capabilities/default.json` (`core:default`, `opener:default`), `capabilities/desktop.json` (`updater:default`). Własne komendy `#[tauri::command]` w Tauri 2 nie wymagają wpisu w capabilities.
- Brak testów w repo; `cargo test` działa, frontend nie ma runnera. Jedyną statyczną bramką frontendu jest `npm run build` (`tsc` strict + `noUnusedLocals`).
- Pipeline wydań gotowy: `.github/workflows/release.yml` (ręczny `workflow_dispatch`, draft), updater przetestowany 0.1.0 → 0.1.1; publikacja draftu wyłącznie ręczna przez użytkownika.

## Desired End State

- Pierwsze uruchomienie (brak pliku sejfu i brak kopii): ekran ustawienia hasła (min. 8 znaków, wpisane dwa razy, ostrzeżenie o braku odzyskiwania). Po ustawieniu powstaje zaszyfrowany plik sejfu i widać pustą powłokę aplikacji.
- Każde kolejne uruchomienie: ekran odblokowania. Poprawne hasło → powłoka aplikacji. Błędne → „Nieprawidłowe hasło”, pole wyczyszczone, kolejna próba bez limitu; plik nietknięty.
- Plik główny uszkodzony, a kopia `.bak` odszyfrowuje się tym samym hasłem → komunikat z datą ostatniej dobrej kopii i przywrócenie dopiero po potwierdzeniu; uszkodzony plik zostaje odłożony pod nową nazwą, nie usunięty.
- Plik główny brakuje, a kopia istnieje → aplikacja nie proponuje nowego hasła, tylko przywrócenie kopii (z hasłem).
- Plik zapisany przez nowszą wersję aplikacji → komunikat „dane z nowszej wersji”, brak otwarcia i brak zapisu.
- W pliku na dysku nie ma żadnego jawnego tekstu danych; bez hasła jest nieczytelny.
- Aktualizacja aplikacji przez updater nie zmienia pliku sejfu (ten sam SHA-256 przed i po) i tym samym hasłem da się go odblokować.

Weryfikacja: `cargo test` (pełna lista przypadków w Testing Strategy), `npm run build`, ręczne scenariusze na wydaniu 0.1.2 i aktualizacji 0.1.2 → 0.1.3 (Faza 4).

### Key Discoveries:

- `src-tauri/src/lib.rs:12` — każda nowa komenda musi trafić do `tauri::generate_handler![...]`, inaczej wywołanie z frontendu pada w czasie działania.
- `context/foundation/infrastructure.md:62,91` — aktualizacje działają tylko do przodu; rejestr ryzyk wymaga kopii przed migracją i testu aktualizacji na realnych danych.
- `context/deployment/deploy-plan.md:12` — `cargo` nie jest w PATH; każde polecenie cargo/tauri trzeba poprzedzić `$env:PATH = "$HOME\.cargo\bin;$env:PATH"` (PowerShell) lub odpowiednikiem w Git Bash.
- `context/deployment/deploy-plan.md:98` — tylko człowiek publikuje draft wydania i obsługuje klucz/sekrety; agent uruchamia workflow wyłącznie po wyraźnym „start”.
- `src-tauri/tauri.conf.json:5` — identyfikator zamrożony; zmiana oznaczałaby „pusty” katalog danych i pozorną utratę historii.

## What We're NOT Doing

- Ręczna blokada („Zablokuj”), automatyczna blokada po bezczynności, zmiana hasła — poza zakresem (decyzja planu; PRD wymaga tylko hasła przy starcie).
- Odzyskiwanie hasła, klucz zapasowy, eksport jawny lub awaryjny — zakazane przez PRD §Access Control. Kopie `.bak` są zaszyfrowane tym samym hasłem i lokalne, więc nie są ścieżką odzyskiwania (rozstrzygnięcie Open Roadmap Question #2).
- Mechanizm migracji formatu i kopia przed migracją — w S-01 tylko pole wersji + odmowa otwarcia pliku z nowszej wersji. Migrator wraz ze znacznikowaną czasem kopią przed migracją powstaje razem z pierwszą realną zmianą formatu (najpewniej S-02).
- Opóźnienia i blokady po błędnych hasłach.
- SQLite/SQLCipher — dane trzymane jako wersjonowana struktura Rusta w jednym pliku.
- Dane domenowe (punkt startowy, tankowania, serwisy) — S-02 i dalej.
- Runner testów frontendu (Vitest itp.).
- Zmiana CSP (`app.security.csp: null`), zmiana zachowania deinstalatora, podpisywanie kodu.
- Panel diagnostyczny po odblokowaniu (ścieżka, wersja formatu).

## Implementation Approach

Logika sejfu powstaje jako czysty moduł Rusta niezależny od Tauri (Faza 1), żeby całą kryptografię i operacje na plikach dało się przetestować `cargo test` na katalogu tymczasowym. Faza 2 owija go w cienką warstwę komend Tauri ze stanem sesji; Faza 3 dokłada bramkę w UI i usuwa demo; Faza 4 dowodzi zachowania na prawdziwej instalacji i prawdziwej aktualizacji.

Wybór zaszyfrowanego pojedynczego pliku zamiast SQLCipher: czyste crate'y Rusta (bez OpenSSL/Perla lokalnie i w CI), błędne hasło to po prostu niezgodny tag AEAD, a zapis to atomowa podmiana pliku. Kosztem jest przepisywanie całego pliku przy każdym zapisie — przy setkach–tysiącach wpisów jednego auta pomijalne.

### Format pliku (kontrakt ładunkowy dla wszystkich kolejnych kawałków)

Plik binarny, nagłówek o stałej długości 59 bajtów, potem szyfrogram:

| Pole | Rozmiar | Wartość |
| --- | --- | --- |
| magic | 4 B | `KMPV` |
| envelope_version | u16 LE | `1` |
| kdf_id | u8 | `1` = Argon2id v0x13 |
| m_cost (KiB) | u32 LE | `65536` (64 MiB) |
| t_cost | u32 LE | `3` |
| p_cost | u32 LE | `1` |
| salt | 16 B | losowa, ustalona przy zakładaniu sejfu |
| nonce | 24 B | losowy, nowy przy każdym zapisie |
| ciphertext | reszta | XChaCha20-Poly1305 (z 16-bajtowym tagiem) |

- Cały nagłówek (59 B) jest danymi uwierzytelnianymi (AAD), więc podmiana parametrów KDF czy wersji też psuje tag.
- Klucz: Argon2id(hasło UTF-8 bez przycinania, salt, parametry z nagłówka) → 32 B.
- Jawny ładunek to JSON wersjonowanej struktury; w S-01: `{"format_version":1,"created_at":<unix s>}`.
- Granice sanity przy parsowaniu nagłówka (np. `m_cost` ≤ 1 GiB, `t_cost` ≤ 10, `p_cost` ≤ 8, `kdf_id` znany) — wartości spoza nich traktujemy jako uszkodzenie, żeby zepsuty nagłówek nie wymusił ogromnej alokacji.

### Pliki w katalogu danych

- `kmplus.vault` — sejf główny
- `kmplus.vault.bak` — poprzednia dobra wersja (odświeżana przy każdym zapisie)
- `kmplus.vault.tmp`, `kmplus.vault.bak.tmp` — pliki robocze atomowego zapisu
- `kmplus.vault.damaged-<unix s>` — odłożony uszkodzony plik główny (nigdy nie usuwany przez aplikację)

## Critical Implementation Details

- **Kolejność zapisu (State sequencing).** Zapis to: (1) jeśli `kmplus.vault` istnieje — skopiuj go do `kmplus.vault.bak.tmp`, `sync_all`, `rename` na `kmplus.vault.bak`; (2) zapisz nowy szyfrogram do `kmplus.vault.tmp`, `sync_all`, `rename` na `kmplus.vault`. Dzięki temu w każdej chwili istnieje kompletny plik główny, a `.bak` zawsze jest poprzednią wersją. Na Windows `std::fs::rename` zastępuje istniejący plik docelowy. Pozostałe po awarii pliki `.tmp` są po prostu nadpisywane przy kolejnym zapisie.
- **Odblokowanie nigdy nie pisze przy porażce.** Ścieżka `unlock` tylko czyta pliki. Jedyne zapisy w S-01 to założenie sejfu i (po potwierdzeniu) przywrócenie kopii.
- **Rozróżnianie błędnego hasła od uszkodzenia.** Nagłówek nieparsowalny (zły magic, za krótki, wartości poza granicami) → plik na pewno uszkodzony. Nagłówek poprawny, tag niezgodny → błędne hasło albo uszkodzenie; wtedy próbujemy `.bak` tym samym hasłem: sukces → „uszkodzony, jest kopia”; porażka lub brak kopii → „Nieprawidłowe hasło”. Nagłówek nieparsowalny i brak kopii → „plik uszkodzony, brak kopii” (bez proponowania nowego hasła).
- **Wydajność KDF i wątki (Performance).** Argon2id przy 64 MiB w buildzie debug Rusta trwa wiele sekund — dodaj `[profile.dev.package.argon2] opt-level = 3` (i dla zależności `blake2`, z której korzysta). Komendy Tauri muszą być `async`, a wyprowadzenie klucza uruchamiane w `tauri::async_runtime::spawn_blocking`, inaczej synchroniczna komenda zamrozi okno. Testy jednostkowe wstrzykują tanie parametry KDF.
- **Higiena pamięci.** Klucz trzymany jako `Zeroizing<[u8; 32]>`; `String` z hasłem zerowany (`zeroize`) zaraz po wyprowadzeniu klucza. Hasło nigdy nie trafia do logów ani komunikatów błędów.

## Phase 1: Rdzeń sejfu (Rust, bez Tauri)

### Overview

Czysty moduł `vault` z formatem pliku, KDF, szyfrowaniem, atomowym zapisem z rotacją `.bak`, otwieraniem z wykrywaniem uszkodzeń, przywracaniem kopii, odmową otwarcia nowszej wersji i regułą długości hasła — w pełni pokryty `cargo test`.

### Changes Required:

#### 1. Zależności i profil

**File**: `src-tauri/Cargo.toml`

**Intent**: Dodać crate'y kryptograficzne i narzędziowe w czystym Ruście oraz przyspieszyć Argon2 w buildach debug.

**Contract**: `[dependencies]`: `argon2`, `chacha20poly1305` (z funkcją `getrandom` do losowania nonce/soli przez `OsRng`), `zeroize`, `thiserror`; `[dev-dependencies]`: `tempfile`. Użyj aktualnych stabilnych wersji i sprawdź ich API (`Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(m,t,p,Some(32)))` + `hash_password_into`; `XChaCha20Poly1305` z `Payload { msg, aad }`). `[profile.dev.package.argon2]` i `[profile.dev.package.blake2]` z `opt-level = 3`.

#### 2. Moduł formatu i kryptografii

**File**: `src-tauri/src/vault/format.rs` (nowy)

**Intent**: Serializacja/parsowanie 59-bajtowego nagłówka z granicami sanity, wyprowadzanie klucza i szyfrowanie/odszyfrowanie ładunku z nagłówkiem jako AAD.

**Contract**: nagłówek i stałe zgodne z tabelą „Format pliku”; `KdfParams { m_cost, t_cost, p_cost }` z `KdfParams::default()` = 65536/3/1; funkcje `derive_key(password, salt, params) -> Zeroizing<[u8;32]>`, `seal(key, header_without_nonce, plaintext) -> Vec<u8>` (losuje nonce), `open(key, bytes) -> Result<Vec<u8>, OpenError>` gdzie `OpenError` rozróżnia `Malformed` (nagłówek), `UnsupportedEnvelope` (envelope_version > 1) i `AuthFailed` (tag).

#### 3. Model danych sejfu

**File**: `src-tauri/src/vault/data.rs` (nowy)

**Intent**: Wersjonowana struktura jawnych danych; jedyne miejsce, które kolejne kawałki będą rozszerzać.

**Contract**: `VaultData { format_version: u32, created_at: u64 }` (serde JSON); `CURRENT_FORMAT_VERSION = 1`; ładunek z `format_version > CURRENT_FORMAT_VERSION` → błąd „nowsza wersja” bez dalszego parsowania pól.

#### 4. Operacje na plikach i stan sejfu

**File**: `src-tauri/src/vault/mod.rs` (nowy; ewentualnie `store.rs` na operacje plikowe)

**Intent**: Publiczne API sejfu operujące na podanym katalogu: wykrywanie stanu startowego, założenie, odblokowanie z rozpoznaniem uszkodzenia, przywrócenie kopii i zapis z rotacją `.bak`.

**Contract**:
- `probe(dir) -> StartupState` — `NeedsSetup` (brak `kmplus.vault` i brak `kmplus.vault.bak`), `Locked` (jest plik główny), `MissingWithBackup { backup_saved_at_ms }` (brak głównego, jest `.bak`; czas = mtime kopii).
- `create(dir, password, params) -> Result<UnlockedVault, VaultError>` — waliduje hasło (`chars().count() >= 8`, bez przycinania), odmawia gdy istnieje plik główny lub `.bak` (`AlreadyExists`), losuje sól, zapisuje atomowo.
- `unlock(dir, password) -> Result<UnlockOutcome, VaultError>` — tylko odczyt; `UnlockOutcome` = `Unlocked(UnlockedVault)` | `WrongPassword` | `DamagedBackupAvailable { backup_saved_at_ms }` | `DamagedNoBackup` | `TooNew`, według reguł z Critical Implementation Details.
- `restore_backup(dir, password) -> Result<UnlockOutcome, VaultError>` — najpierw odszyfrowuje `.bak` (porażka → `WrongPassword`, nic nie zmienia na dysku); przy sukcesie odkłada istniejący plik główny jako `kmplus.vault.damaged-<unix s>`, kopiuje `.bak` → `.tmp` → `rename` na plik główny, zwraca `Unlocked`. `.bak` zostaje.
- `UnlockedVault { dir, key, salt, params, data }` z metodą `save(&mut self) -> Result<(), VaultError>` realizującą kolejność zapisu z Critical Implementation Details (nowy nonce, ta sama sól i parametry). W S-01 wywoływana przez `create` i testy; to API dla S-02+.
- `VaultError` (thiserror) dla błędów IO i `PasswordTooShort`, `AlreadyExists`; komunikaty bez hasła.

#### 5. Testy jednostkowe

**File**: `src-tauri/src/vault/tests.rs` (nowy, `#[cfg(test)]`) lub moduły `tests` w plikach powyżej

**Intent**: Pokryć każdy przypadek z Testing Strategy na `tempfile::TempDir` z tanimi parametrami KDF (wstrzykiwanymi; produkcyjne domyślne tylko w jednym teście potwierdzającym wartości nagłówka).

**Contract**: nazwane testy wg listy w Testing Strategy → Unit Tests.

#### 6. Rejestracja modułu

**File**: `src-tauri/src/lib.rs`

**Intent**: Dołączyć `mod vault;` (bez zmian w `run()` w tej fazie).

**Contract**: moduł kompiluje się bez ostrzeżeń o nieużywanym kodzie zablokowanych przez build (dopuszczalne `#[allow(dead_code)]` do Fazy 2 albo `pub` API).

### Success Criteria:

#### Automated Verification:

- Testy sejfu przechodzą: `cargo test vault` w `src-tauri/`
- Projekt Rusta się kompiluje: `cargo check` w `src-tauri/`
- Brak znanych podatności w nowych zależnościach: `cargo audit` w `src-tauri/`

#### Manual Verification:

- Przegląd listy testów potwierdza pokrycie każdego punktu z Testing Strategy → Unit Tests

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 2: Komendy Tauri i stan sesji

### Overview

Cienka warstwa komend nad modułem `vault`: stan sesji w `app.manage`, ścieżka z `app_data_dir`, asynchroniczne komendy z KDF poza głównym wątkiem, usunięcie `greet`.

### Changes Required:

#### 1. Stan i komendy

**File**: `src-tauri/src/vault/commands.rs` (nowy) i `src-tauri/src/lib.rs`

**Intent**: Wystawić frontendowi cztery komendy i trzymać odblokowany sejf w pamięci do zamknięcia aplikacji.

**Contract**:
- Stan: `VaultSession(Mutex<Option<UnlockedVault>>)` rejestrowany przez `.manage(...)` w `run()`; katalog z `app.path().app_data_dir()` (tworzony, jeśli brak).
- Komendy (`#[tauri::command] async`, KDF w `tauri::async_runtime::spawn_blocking`), zwracające `Result<T, String>` (Err = błąd IO z czytelnym komunikatem bez hasła):
  - `vault_status() -> VaultStatus` = `{ kind: "needsSetup" } | { kind: "locked" } | { kind: "missingWithBackup", backupSavedAt: number } | { kind: "unlocked" }` (ostatnie, gdy sesja już odblokowana — np. po przeładowaniu webview).
  - `vault_setup(password: string) -> SetupResult` = `{ kind: "ok" } | { kind: "passwordTooShort" } | { kind: "alreadyExists" }`; przy `ok` sesja odblokowana.
  - `vault_unlock(password: string) -> UnlockResult` = `{ kind: "unlocked" } | { kind: "wrongPassword" } | { kind: "damagedBackupAvailable", backupSavedAt: number } | { kind: "damagedNoBackup" } | { kind: "tooNew" }`.
  - `vault_restore_backup(password: string) -> UnlockResult` — używana zarówno po `damagedBackupAvailable`, jak i przy `missingWithBackup`.
- Serde: `#[serde(tag = "kind", rename_all = "camelCase")]`, pola w camelCase; czasy w milisekundach epoki.
- `generate_handler![vault_status, vault_setup, vault_unlock, vault_restore_backup]`; funkcja `greet` usunięta.

#### 2. Rejestr nazw nośnych

**File**: `docs/reference/contract-surfaces.md` (nowy)

**Intent**: Zapisać nazwy, których zmiana psuje dane lub frontend: nazwy plików sejfu, magic i wersje (envelope, format_version), parametry domyślne KDF, nazwy komend i kształty `kind`.

**Contract**: krótka tabela nazwa → gdzie zdefiniowana → konsekwencja zmiany.

### Success Criteria:

#### Automated Verification:

- Projekt Rusta się kompiluje: `cargo check` w `src-tauri/`
- Testy nadal przechodzą: `cargo test` w `src-tauri/`
- `greet` nie występuje już w backendzie: `grep -r greet src-tauri/src` nic nie zwraca

#### Manual Verification:

- `docs/reference/contract-surfaces.md` wymienia wszystkie cztery komendy, nazwy plików sejfu i wersje formatu

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 3: Bramka w UI i pusta powłoka

### Overview

Frontend pokazuje odpowiedni ekran zależnie od stanu sejfu, a po odblokowaniu pustą powłokę kmPlus. Demo szablonu znika, baner aktualizacji zostaje widoczny niezależnie od stanu bramki.

### Changes Required:

#### 1. Typowane wywołania backendu

**File**: `src/vault/api.ts` (nowy)

**Intent**: Jedno miejsce z typami TS odpowiadającymi kształtom z Fazy 2 i opakowaniami `invoke`.

**Contract**: typy `VaultStatus`, `SetupResult`, `UnlockResult` (unie z polem `kind`) i funkcje `vaultStatus()`, `vaultSetup(password)`, `vaultUnlock(password)`, `vaultRestoreBackup(password)`.

#### 2. Bramka i ekrany

**File**: `src/vault/VaultGate.tsx` (nowy; ekrany mogą być osobnymi plikami w `src/vault/`)

**Intent**: Maszyna stanów: ładowanie → konfiguracja hasła | odblokowanie | brak pliku z kopią | uszkodzony z kopią | uszkodzony bez kopii | nowsza wersja | odblokowano (renderuje `children`).

**Contract** (teksty po polsku):
- Konfiguracja: dwa pola hasła, przycisk aktywny dopiero przy ≥ 8 znakach i zgodności pól; stałe ostrzeżenie: hasła nie da się odzyskać, jego utrata oznacza utratę wszystkich danych, zapisz je w menedżerze haseł.
- Odblokowanie: jedno pole, Enter wysyła; w trakcie KDF przycisk wyłączony z tekstem „Odblokowywanie…”; `wrongPassword` → „Nieprawidłowe hasło”, pole wyczyszczone, fokus wraca do pola.
- `damagedBackupAvailable`: komunikat, że plik danych jest uszkodzony, a ostatnia dobra kopia pochodzi z <data i godzina lokalnie>; przyciski „Przywróć kopię” (wywołuje `vaultRestoreBackup` z tym samym hasłem trzymanym w stanie komponentu) i „Anuluj” (powrót do odblokowania).
- `missingWithBackup`: komunikat o braku pliku głównego i dacie kopii, pole hasła + „Przywróć kopię”; brak opcji ustawienia nowego hasła.
- `damagedNoBackup`: komunikat o uszkodzonym pliku bez kopii i ścieżce katalogu danych; brak opcji ustawienia nowego hasła.
- `tooNew`: komunikat, że dane pochodzą z nowszej wersji aplikacji i trzeba ją zaktualizować.
- Błąd `Err` z komendy: komunikat ogólny z treścią błędu i możliwość ponowienia.
- Hasło czyszczone ze stanu komponentu po zakończeniu przepływu.

#### 3. Powłoka i usunięcie demo

**File**: `src/App.tsx`, `src/App.css`, `src/assets/react.svg`, `public/vite.svg`, `public/tauri.svg`, `index.html`

**Intent**: `App` renderuje baner aktualizacji (bez zmian zachowania) oraz `<VaultGate>` z pustą powłoką: nagłówek „kmPlus” i tekst zastępczy „Brak wpisów”. Usunąć formularz `greet`, logotypy, ich style i nieużywane zasoby; jeśli `index.html` odwołuje się do `vite.svg` jako favicon, usunąć lub podmienić odwołanie.

**Contract**: brak importu `invoke("greet")`; `npm run build` przechodzi przy `noUnusedLocals`.

### Success Criteria:

#### Automated Verification:

- Frontend się buduje (tsc strict + vite): `npm run build`
- Brak pozostałości demo: `grep -rE "greet|reactLogo" src` nic nie zwraca
- Brak podatności w zależnościach JS: `npm audit`

#### Manual Verification:

- `npm run tauri dev` na czystym katalogu danych: ekran ustawienia hasła; za krótkie lub niezgodne hasła blokują przycisk; po ustawieniu widać powłokę „kmPlus / Brak wpisów”
- Ponowne uruchomienie: ekran odblokowania; błędne hasło → „Nieprawidłowe hasło” i wyczyszczone pole; poprawne → powłoka; okno nie zamarza podczas odblokowywania
- Scenariusz uszkodzenia: po zamknięciu aplikacji skopiować `kmplus.vault` na `kmplus.vault.bak`, zmienić jeden bajt w `kmplus.vault`; przy starcie i poprawnym haśle pojawia się komunikat z datą kopii; „Przywróć kopię” odblokowuje, w katalogu jest `kmplus.vault.damaged-<…>`
- Scenariusz braku pliku: przenieść `kmplus.vault` poza katalog (zostawić `.bak`); start pokazuje przywracanie kopii, nie ustawianie hasła

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 4: Wydanie i dowód aktualizacji

### Overview

Dowód na prawdziwej instalacji: wydanie 0.1.2 z sejfem przez istniejący pipeline, ustawienie hasła na zainstalowanej kopii, potem wydanie 0.1.3 i aktualizacja w miejscu bez naruszenia danych. Wszystkie bramki ręczne z `context/deployment/deploy-plan.md` obowiązują (publikacja draftu i start workflow tylko za zgodą/ręką użytkownika).

### Changes Required:

#### 1. Wersja 0.1.2

**File**: `src-tauri/tauri.conf.json`, `package.json`, `src-tauri/Cargo.toml` (+ `Cargo.lock`, `package-lock.json`)

**Intent**: Podbić wersję do 0.1.2 we wszystkich trzech miejscach, commit, push na `main`, uruchomienie `release.yml` po wyraźnym „start” użytkownika, weryfikacja draftu (`gh release view app-v0.1.2`: `*-setup.exe`, `.sig`, `latest.json`); publikacja ręczna przez użytkownika.

**Contract**: trzy pliki wersji zgodne; tag `app-v0.1.2`.

#### 2. Wersja 0.1.3

**File**: te same trzy pliki

**Intent**: Po ręcznych testach na 0.1.2 podbić wersję do 0.1.3 bez innych zmian kodu i powtórzyć rytuał wydania, żeby aktualizacja 0.1.2 → 0.1.3 sprawdziła zachowanie danych.

**Contract**: tag `app-v0.1.3`.

#### 3. Zapis przebiegu

**File**: `context/changes/password-unlocked-vault/change.md` (sekcja `## Notes`)

**Intent**: Ślad dowodowy: numery runów obu wydań oraz SHA-256 `kmplus.vault` przed i po aktualizacji 0.1.2 → 0.1.3.

**Contract**: dopisane wiersze w `## Notes`; brak zmian w kodzie.

### Success Criteria:

#### Automated Verification:

- Workflow wydania 0.1.2 kończy się sukcesem: `gh run watch` na runie `release.yml`
- Draft `app-v0.1.2` zawiera instalator, `.sig` i `latest.json`: `gh release view app-v0.1.2`
- Workflow wydania 0.1.3 kończy się sukcesem i draft `app-v0.1.3` jest kompletny: `gh release view app-v0.1.3`

#### Manual Verification:

- Zainstalowana 0.1.1 aktualizuje się do 0.1.2 przez baner; pierwsze uruchomienie pokazuje ustawianie hasła; plik `%APPDATA%\io.github.m-swietek.kmplus\kmplus.vault` istnieje i nie zawiera jawnego tekstu (np. brak `format_version` i `created_at` w bajtach pliku)
- Na 0.1.2: zamknięcie i ponowne otwarcie wymaga hasła; błędne hasło daje komunikat; poprawne odblokowuje
- Aktualizacja 0.1.2 → 0.1.3 przez baner: SHA-256 `kmplus.vault` identyczny przed i po aktualizacji, a to samo hasło odblokowuje aplikację (bez ekranu ustawiania hasła)

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Testing Strategy

### Unit Tests:

Wszystkie w `cargo test` na `tempfile::TempDir`, tanie parametry KDF (poza testem wartości domyślnych):

- `create_then_unlock_roundtrip` — założenie, odblokowanie, te same `VaultData`
- `wrong_password_returns_wrong_password_and_leaves_files_untouched` — bajty i mtime plików bez zmian
- `password_shorter_than_8_chars_rejected` oraz granica: 8 znaków (w tym znaki wielobajtowe, np. `ąęółżźćń`) przechodzi, 7 nie; brak przycinania spacji
- `create_refuses_when_vault_or_backup_exists`
- `file_contains_no_plaintext` — znacznik w ładunku nie występuje w bajtach pliku
- `tampered_ciphertext_byte_fails` i `tampered_header_byte_fails` (AAD)
- `truncated_file_is_malformed`, `bad_magic_is_malformed`, `absurd_kdf_params_are_malformed`
- `save_rotates_previous_version_into_bak` — po dwóch zapisach `.bak` odszyfrowuje się do poprzedniej wersji danych
- `each_save_uses_fresh_nonce`
- `damaged_main_with_good_backup_reports_backup_available`
- `wrong_password_with_backup_present_reports_wrong_password`
- `malformed_main_without_backup_reports_damaged_no_backup`
- `restore_backup_sets_aside_damaged_main_and_unlocks` — powstaje `kmplus.vault.damaged-*`, `.bak` zostaje
- `restore_backup_with_wrong_password_changes_nothing`
- `probe_states` — `NeedsSetup`, `Locked`, `MissingWithBackup`
- `newer_envelope_version_is_too_new` i `newer_format_version_is_too_new` — bez zapisu na dysk
- `default_kdf_params_written_to_header` — 65536/3/1

### Integration Tests:

- Brak automatycznych testów integracyjnych Tauri/UI w tym kawałku (decyzja planu); ścieżkę komend i UI sprawdzają ręczne scenariusze Fazy 3 i 4.

### Manual Testing Steps:

1. `npm run tauri dev` przy pustym katalogu danych → ustawienie hasła → powłoka.
2. Restart → błędne hasło → komunikat; poprawne → powłoka.
3. Scenariusz uszkodzenia (kopia ręczna `.bak` + zmiana bajtu) → przywrócenie po potwierdzeniu.
4. Scenariusz brakującego pliku → przywrócenie zamiast nowego hasła.
5. Instalacja 0.1.2, potem aktualizacja do 0.1.3 z porównaniem SHA-256 pliku sejfu.

## Performance Considerations

- Odblokowanie ≈ 0,5–1 s w buildzie release (Argon2id 64 MiB / 3 iteracje); w scenariuszu „sprawdź kopię” dwa wyprowadzenia klucza (~2×). Parametry siedzą w nagłówku, więc można je później podnieść bez zmiany formatu.
- Zapis przepisuje cały plik; przy wielkości danych jednego auta bez znaczenia.

## Migration Notes

- Brak istniejących danych do migracji: zainstalowane 0.1.0/0.1.1 nie przechowują niczego.
- Od 0.1.2 plik sejfu jest jedyną kopią historii. Pierwsza zmiana `format_version` (najpewniej S-02) musi dodać migrator oraz znacznikowaną czasem kopię przed migracją (`infrastructure.md:91`) — ta część decyzji o kopiach jest świadomie przesunięta do tamtej zmiany. Pola dodawane jako opcjonalne z `#[serde(default)]` nie wymagają zmiany wersji, ale starsza aplikacja otwierająca nowszy plik musi trafić w `TooNew`, więc S-02 powinien świadomie zdecydować o podbiciu wersji.

## References

- Roadmap: `context/foundation/roadmap.md` (S-01, Open Roadmap Question #2)
- PRD: `context/foundation/prd.md` (FR-001, §Access Control, §Guardrails, §Non-Functional Requirements)
- Ryzyka wdrożenia: `context/foundation/infrastructure.md:62,91`
- Rytuał wydań: `context/deployment/deploy-plan.md` (kroki 9, 11–13)
- Punkt rejestracji komend: `src-tauri/src/lib.rs:12`
- Baner aktualizacji do zachowania: `src/App.tsx:13-49`

## Progress

> Convention: `- [ ]` pending, `- [x]` done. Append ` — <commit sha>` when a step lands. Do not rename step titles. See `references/progress-format.md`.

### Phase 1: Rdzeń sejfu (Rust, bez Tauri)

#### Automated

- [x] 1.1 Testy sejfu przechodzą: `cargo test vault` w `src-tauri/` — ebab2cf
- [x] 1.2 Projekt Rusta się kompiluje: `cargo check` w `src-tauri/` — ebab2cf
- [x] 1.3 Brak znanych podatności w nowych zależnościach: `cargo audit` w `src-tauri/` — ebab2cf

#### Manual

- [x] 1.4 Przegląd listy testów potwierdza pokrycie każdego punktu z Testing Strategy → Unit Tests — ebab2cf

### Phase 2: Komendy Tauri i stan sesji

#### Automated

- [x] 2.1 Projekt Rusta się kompiluje: `cargo check` w `src-tauri/` — 4ac6b6e
- [x] 2.2 Testy nadal przechodzą: `cargo test` w `src-tauri/` — 4ac6b6e
- [x] 2.3 `greet` nie występuje już w backendzie: `grep -r greet src-tauri/src` nic nie zwraca — 4ac6b6e

#### Manual

- [x] 2.4 `docs/reference/contract-surfaces.md` wymienia wszystkie cztery komendy, nazwy plików sejfu i wersje formatu — 4ac6b6e

### Phase 3: Bramka w UI i pusta powłoka

#### Automated

- [x] 3.1 Frontend się buduje (tsc strict + vite): `npm run build`
- [x] 3.2 Brak pozostałości demo: `grep -rE "greet|reactLogo" src` nic nie zwraca
- [x] 3.3 Brak podatności w zależnościach JS: `npm audit`

#### Manual

- [x] 3.4 `npm run tauri dev` na czystym katalogu danych: ekran ustawienia hasła; za krótkie lub niezgodne hasła blokują przycisk; po ustawieniu widać powłokę „kmPlus / Brak wpisów”
- [x] 3.5 Ponowne uruchomienie: ekran odblokowania; błędne hasło → „Nieprawidłowe hasło” i wyczyszczone pole; poprawne → powłoka; okno nie zamarza podczas odblokowywania
- [x] 3.6 Scenariusz uszkodzenia: po zamknięciu aplikacji skopiować `kmplus.vault` na `kmplus.vault.bak`, zmienić jeden bajt w `kmplus.vault`; przy starcie i poprawnym haśle pojawia się komunikat z datą kopii; „Przywróć kopię” odblokowuje, w katalogu jest `kmplus.vault.damaged-<…>`
- [x] 3.7 Scenariusz braku pliku: przenieść `kmplus.vault` poza katalog (zostawić `.bak`); start pokazuje przywracanie kopii, nie ustawianie hasła

### Phase 4: Wydanie i dowód aktualizacji

#### Automated

- [ ] 4.1 Workflow wydania 0.1.2 kończy się sukcesem: `gh run watch` na runie `release.yml`
- [ ] 4.2 Draft `app-v0.1.2` zawiera instalator, `.sig` i `latest.json`: `gh release view app-v0.1.2`
- [ ] 4.3 Workflow wydania 0.1.3 kończy się sukcesem i draft `app-v0.1.3` jest kompletny: `gh release view app-v0.1.3`

#### Manual

- [ ] 4.4 Zainstalowana 0.1.1 aktualizuje się do 0.1.2 przez baner; pierwsze uruchomienie pokazuje ustawianie hasła; plik `%APPDATA%\io.github.m-swietek.kmplus\kmplus.vault` istnieje i nie zawiera jawnego tekstu (np. brak `format_version` i `created_at` w bajtach pliku)
- [ ] 4.5 Na 0.1.2: zamknięcie i ponowne otwarcie wymaga hasła; błędne hasło daje komunikat; poprawne odblokowuje
- [ ] 4.6 Aktualizacja 0.1.2 → 0.1.3 przez baner: SHA-256 `kmplus.vault` identyczny przed i po aktualizacji, a to samo hasło odblokowuje aplikację (bez ekranu ustawiania hasła)
