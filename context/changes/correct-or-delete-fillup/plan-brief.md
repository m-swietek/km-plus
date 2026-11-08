# Poprawka i usunięcie tankowania — Plan Brief

> Full plan: `context/changes/correct-or-delete-fillup/plan.md`

## What & Why

S-04 roadmapy (US-01, FR-004, FR-005): błędny wpis tankowania można poprawić albo usunąć, a spalanie obu paliw przelicza się ponownie. To pierwszy destrukcyjny zapis na jedynej kopii historii, a przeliczenie po poprawce musi dawać dokładnie to, co wpisanie danych od nowa — błędna liczba jest gorsza niż brak liczby.

## Starting Point

Wyniki są czystą funkcją punktu startowego i całej listy (`compute`, `fuel_reports` w `src-tauri/src/fuel/mod.rs`), więc przeliczenie jest „za darmo”; wpisy mają trwałe `id` od S-02. Brakuje walidacji poprawki (dzisiejsza odrzuciłaby wpis zapisany bez zmian jako kolizję z samym sobą), komend edycji i usuwania oraz akcji w UI.

## Desired End State

Każdy wiersz historii ma „Edytuj” i „Usuń”. „Edytuj” ładuje wpis do wspólnego formularza („Popraw tankowanie”, „Zapisz zmiany” / „Anuluj”), wszystkie pola łącznie z paliwem są edytowalne, a wpis zachowuje `id`. „Usuń” wymaga drugiego kliknięcia w wierszu. Jeden panel pod formularzem mówi, co się stało („Zapisano” / „Poprawiono” z przeliczonym wynikiem / „Usunięto”), a karty i historie od razu pokazują przeliczone liczby. Wydane jako 0.1.7.

## Key Decisions Made

| Decision | Choice | Why (1 sentence) | Source |
| --- | --- | --- | --- |
| Ochrona usunięcia | Dwuetapowe potwierdzenie w wierszu; bez cofania i kosza | Chroni przed przypadkiem bez zmiany formatu sejfu i bez wtyczki dialog. | Plan |
| Miejsce edycji | Wspólny formularz w trybie edycji | Jedno miejsce parsowania, strażników i komunikatów odrzuceń. | Plan |
| Zmiana paliwa | Wszystkie pola edytowalne, `id` zachowane, przeliczane oba paliwa | Najgroźniejszą literówkę poprawia jedna akcja, nie usunięcie + ponowne wpisanie. | Plan |
| Informacja po akcji | Jeden panel statusu ostatniej akcji (Zapisano / Poprawiono + wynik / Usunięto) | Wykorzystuje istniejący panel wyniku i nigdy nie zostawia nieaktualnej liczby. | Plan |
| Walidacja poprawki | Reguły dodawania wobec wszystkich wpisów poza edytowanym; usunięcie bez walidacji | Reguły są parami, więc usunięcie nie łamie spójności, a poprawka bez zmian jest przyjmowana. | Plan |
| Format sejfu | Bez zmian, bez migracji | Usuwanie ostateczne nie wymaga nowego pola. | Plan |
| Drobne zachowania UI | „Edytuj” zastępuje szkic; po zapisie/anulowaniu powrót do dodawania; usunięcie edytowanego wpisu kończy edycję | Decyzje agenta, niezakwestionowane przez użytkownika. | Plan |
| Stan wspólny w UI | Hook `useFillUpEditing` w `src/fuel/`, `AppShell` tylko woła hook | Minimalny konflikt z zakładkami „Paliwo / Serwisy” z S-05. | Plan |
| Wydanie | 0.1.7 (albo kolejny numer po S-05) rytuałem z S-02/S-03 | Poprawki od razu na prawdziwym dzienniku. | Plan |

## Scope

**In scope:** domenowe `replace_fill_up` / `remove_fill_up` z testami równoważności „poprawka/usunięcie = wpisanie od nowa”; komendy `update_fill_up` / `delete_fill_up` z testami kształtu i trwałości, `lib.rs`, `contract-surfaces.md`, `api.ts`; tryb edycji formularza, akcje wiersza z potwierdzeniem, panel statusu; wydanie z dowodem aktualizacji.

**Out of scope:** cofanie i kosz, zmiana formatu sejfu, edycja w wierszu i modal, natywne okno potwierdzenia, wartości „przed → po”, zachowanie szkicu nowego wpisu, usuwanie wielu wpisów, edycja punktu startowego / wpisów przebiegu / serwisów, zmiana reguły spalania.

## Architecture / Approach

`replace_fill_up` / `remove_fill_up` (czysta domena, walidacja `validate_fill_up` bez edytowanego wpisu) → `apply_update` / `apply_delete` w `commands.rs` (wzorzec `apply_fill_up`, zapis przez `commit`, wynik z `compute`) → `updateFillUp` / `deleteFillUp` w `api.ts` → hook `useFillUpEditing` (wpis w edycji, potwierdzany wiersz, komunikat) → `FillUpForm` (tryb edycji), `FillUpHistory` (akcje), `FillUpResultPanel` (komunikat) → istniejące `fetchOverview` odświeża karty i historie.

## Phases at a Glance

| Phase | What it delivers | Key risk |
| --- | --- | --- |
| 1. Domena | Poprawka i usunięcie z testami na próbce wyroczni | Walidacja poprawki wobec samego siebie |
| 2. Kontrakt | `update_fill_up` / `delete_fill_up`, testy JSON i `TempDir`, rejestr, typy TS | Rozjazd DTO ↔ `api.ts`; komenda niezarejestrowana w `generate_handler!` |
| 3. UI | Tryb edycji, potwierdzenie w wierszu, panel statusu | Test ręczny zapisuje dane — prawdziwy sejf musi być odłożony i przywrócony |
| 4. Wydanie 0.1.7 | Release + dowód aktualizacji + próba na wpisie testowym | Pominięty zapis dowodów; kolejność wydań z S-05 |

**Prerequisites:** S-03 wydane (0.1.6 zainstalowane z prawdziwymi danymi); `gh` z dostępem do repo.
**Estimated effort:** ~2–3 sesje w 4 fazach.

## Open Risks & Assumptions

- `npm run tauri dev` pisze do tego samego katalogu co zainstalowana aplikacja; przed testem Fazy 3 prawdziwe pliki idą na bok, po teście wracają z weryfikacją SHA-256 (lekcja z F2 w S-03).
- Usunięcie jest ostateczne; `kmplus.vault.bak` trzyma stan sprzed usunięcia tylko do następnego zapisu i nie jest funkcją „cofnij”.
- Równoległe S-05 dotyka tych samych plików: `src/AppShell.tsx`, `src-tauri/src/lib.rs` (`generate_handler!`), `docs/reference/contract-surfaces.md`, pliki wersji (`src/App.css` zmienia tylko S-04 — S-05 ma własny `src/service/service.css`). Jeśli S-05 wyjdzie pierwsze, S-04 dostaje kolejny numer wersji, a format sejfu jest ten, który S-05 zostawi (S-04 go nie zmienia).
- Ręczna weryfikacja Fazy 3 i Faza 4 nie mogą nakładać się w czasie z tymi samymi fazami S-05 (wspólny katalog danych i port 1420); wydania po kolei, zalecane S-04 pierwsze. Potwierdzenie usunięcia („Usunąć …? [Usuń] [Anuluj]”) i komunikat „Ten wpis już nie istnieje.” są wspólne z S-05.
- Sprawdzenie na prawdziwych danych w Fazie 4 robione na dolewce testowej, żeby nie zmieniać prawdziwej historii.

## Success Criteria (Summary)

- Po poprawce lub usunięciu dowolnego wpisu karty i historie obu paliw pokazują te same liczby, co lista wpisana od nowa.
- Wpisu nie da się usunąć jednym kliknięciem, a poprawka bez zmian nigdy nie jest odrzucana.
- Po aktualizacji do nowej wersji prawdziwy dziennik otwiera się bez zmiany pliku, a poprawka i usunięcie działają na nim.
