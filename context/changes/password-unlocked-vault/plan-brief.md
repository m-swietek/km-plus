# Hasło odblokowuje zaszyfrowane dane (S-01) — Plan Brief

> Full plan: `context/changes/password-unlocked-vault/plan.md`

## What & Why

Dokładamy warstwę danych kmPlus: jeden zaszyfrowany plik w katalogu danych aplikacji, odszyfrowywany kluczem z hasła użytkownika (FR-001). Musi to być pierwsze, bo każdy kolejny kawałek roadmapy zapisuje dane, a szyfrowanie dołożone później wymagałoby konwersji jedynej kopii historii.

## Starting Point

Nietknięty scaffold Tauri 2 + React: komenda `greet`, demo z logotypami i działający baner aktualizacji (`src/App.tsx`). Brak składowania, kryptografii i testów. Pipeline wydań i updater działają (0.1.0 → 0.1.1 przetestowane).

## Desired End State

Przy pierwszym uruchomieniu użytkownik ustawia hasło, przy każdym kolejnym nim odblokowuje aplikację i widzi pustą powłokę „kmPlus”. Plik na dysku jest nieczytelny bez hasła; błędne hasło daje jasny komunikat i niczego nie zapisuje; uszkodzony lub brakujący plik jest — za zgodą — przywracany z ostatniej dobrej kopii, nigdy po cichu zastępowany pustym sejfem. Aktualizacja aplikacji nie zmienia pliku (ten sam SHA-256) i to samo hasło dalej działa.

## Key Decisions Made

| Decision | Choice | Why (1 sentence) |
| --- | --- | --- |
| Składowanie | Jeden plik: nagłówek + XChaCha20-Poly1305, klucz z Argon2id, zapis atomowy | Czysty Rust (bez OpenSSL/Perla w CI), błędne hasło = niezgodny tag, nigdy częściowy zapis. |
| Kopie | `.bak` przy każdym zapisie; kopia przed migracją razem z pierwszym migratorem | Chroni przed złym zapisem teraz; kopie są zaszyfrowane, więc nie są ścieżką odzyskiwania (zamyka Open Roadmap Question #2). |
| Reguły hasła | Min. 8 znaków, wpisane dwa razy, ostrzeżenie o braku odzyskiwania | Literówka przy ustawianiu = utrata danych na zawsze. |
| Błędne hasło | Komunikat, bez limitu prób i opóźnień | Model zagrożeń PRD to przypadkowy wgląd, a Argon2id i tak kosztuje ~0,5 s na próbę. |
| Sesja | Tylko odblokowanie przy starcie; bez blokady, auto-blokady i zmiany hasła | Dokładnie FR-001, najmniejsza powierzchnia. |
| Uszkodzony plik | Wykrycie przez próbę `.bak`, przywrócenie po potwierdzeniu, uszkodzony odłożony | Brak cichej utraty danych; nic nie jest kasowane. |
| Pierwsze uruchomienie | Tylko gdy brak pliku i brak kopii; inaczej przywracanie | Nowe hasło nie może przykryć istniejącej historii. |
| Koszt KDF | ~0,5–1 s (64 MiB, 3 iteracje), parametry w nagłówku | Mocne przy niezauważalnym czasie startu; można podnieść później. |
| Po odblokowaniu | Pusta powłoka, demo usunięte | Czysta baza dla S-02, bez kodu do wyrzucenia. |
| Migracje | Tylko pole wersji + odmowa otwarcia nowszego pliku | Mniej kodu teraz; migrator powstaje z pierwszą realną zmianą formatu. |
| Weryfikacja | `cargo test` dla wszystkich przypadków + ręczny test aktualizacji | Automatyczne pokrycie kryptografii/IO i jeden prawdziwy dowód ścieżki aktualizacji. |

## Scope

**In scope:**
- Moduł `vault` w Ruście (format, KDF, szyfrowanie, zapis atomowy z `.bak`, wykrywanie uszkodzeń, przywracanie, odmowa nowszej wersji)
- Komendy `vault_status`, `vault_setup`, `vault_unlock`, `vault_restore_backup`; usunięcie `greet`
- Ekrany: ustawienie hasła, odblokowanie, uszkodzony/brakujący plik, nowsza wersja; pusta powłoka
- `docs/reference/contract-surfaces.md` z nazwami nośnymi
- Wydania 0.1.2 i 0.1.3 jako dowód aktualizacji

**Out of scope:**
- Blokada ręczna/automatyczna, zmiana hasła, odzyskiwanie hasła, eksport
- Migrator formatu i kopia przed migracją (przy pierwszej zmianie formatu)
- Dane domenowe (S-02+), SQLite, runner testów frontendu, CSP, podpisywanie kodu

## Architecture / Approach

Frontend (`src/vault/VaultGate.tsx`) pyta backend o stan i renderuje właściwy ekran. Asynchroniczne komendy Tauri (`src-tauri/src/vault/commands.rs`) wyprowadzają klucz w `spawn_blocking` i trzymają odblokowany sejf w `Mutex` do zamknięcia aplikacji. Czysty moduł `vault` operuje na `%APPDATA%\io.github.m-swietek.kmplus\kmplus.vault` (+ `.bak`, `.tmp`, `.damaged-<ts>`). Nagłówek 59 B (magic `KMPV`, wersja, parametry Argon2id, sól, nonce) jest danymi uwierzytelnianymi; ładunek to JSON `VaultData { format_version, created_at }`.

## Phases at a Glance

| Phase | What it delivers | Key risk |
| --- | --- | --- |
| 1. Rdzeń sejfu | Moduł `vault` + pełna lista testów `cargo test` | Błędna kolejność zapisu lub rozróżnienia „hasło vs uszkodzenie” |
| 2. Komendy Tauri | 4 komendy, stan sesji, rejestr nazw, bez `greet` | KDF na głównym wątku zamraża okno |
| 3. Bramka w UI | Ekrany stanów sejfu, pusta powłoka, demo usunięte | Przepływ przywracania myli użytkownika |
| 4. Wydanie i aktualizacja | 0.1.2 i 0.1.3 opublikowane, dowód SHA-256 | Bramki ręczne (publikacja, start workflow) |

**Prerequisites:** cargo w PATH (`$HOME\.cargo\bin`), `gh` zalogowany, zainstalowana 0.1.1, sekrety podpisu updatera w repo.
**Estimated effort:** ~4–5 sesji w 4 fazach (Faza 4 to głównie czekanie na CI i ręczne testy).

## Open Risks & Assumptions

- Rust i kryptografia to główny blocker umiejętności (roadmap `top_blocker: skills`) — łagodzone przez gotowe crate'y i rozbudowane testy w Fazie 1.
- Zakładamy, że `std::fs::rename` na Windows atomowo zastępuje plik docelowy na tym samym woluminie.
- Pierwsza zmiana formatu (S-02) musi dodać migrator i kopię przed migracją — inaczej ryzyko z `infrastructure.md:91` wraca.
- Utrata hasła = utrata całej historii (decyzja PRD, nie luka).

## Success Criteria (Summary)

- Ustawiam hasło raz, potem odblokowuję nim aplikację; bez hasła plik na dysku jest nieczytelny.
- Błędne hasło, uszkodzony lub brakujący plik nigdy nie kończą się utratą danych ani cichym nowym sejfem.
- Po aktualizacji aplikacji dane są nietknięte i to samo hasło działa.
