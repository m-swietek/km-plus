# Rejestr serwisów — Implementation Plan

## Overview

S-05 z roadmapy (US-02, FR-008, FR-009): użytkownik w nowej zakładce „Serwisy” dodaje zaplanowany przegląd lub wymianę z progiem przebiegu, progiem daty albo obydwoma, opcjonalnie oznacza wpis jako cykliczny (z interwałem dla każdego zadanego progu), a potem edytuje lub usuwa wpis. Oznaczenie „cykliczny” niczego nie tworzy (kolejne wystąpienie to S-08), aplikacja nie ocenia jeszcze, czy próg został osiągnięty (przypomnienia to S-07). Rejestr wymaga nowego pola w sejfie, więc zmiana jest pierwszą od S-02 migracją formatu (v2 → v3) na prawdziwych danych i kończy się osobnym wydaniem.

## Current State Analysis

Na commicie `22a1164` (wersja 0.1.6):

- Ładunek sejfu to v2: `format_version`, `created_at`, `start_point`, `fill_ups`, `next_fill_up_id` — bez `#[serde(default)]`, brak pola = uszkodzenie (`src-tauri/src/vault/data.rs:38-54`). Nie ma miejsca na serwisy.
- Mechanizm migracji istnieje: `decode` czyta najpierw samą wersję, starszą migruje w pamięci, nowszą odrzuca jako `TooNew` (`data.rs:98-117`); pierwszy zapis po migracji robi kopię `kmplus.vault.pre-v<CURRENT_FORMAT_VERSION>-<s>` przed rotacją `.bak`, odblokowanie niczego nie pisze (`vault/mod.rs:118-159`, `:211-238`).
- Wzorzec domeny i kontraktu z paliw: czysta walidacja bez IO (`src-tauri/src/fuel/mod.rs:113-178`), `id` + licznik `next_*_id` bez ponownego użycia (`data.rs:51-53`), DTO `kind`-tagowane w camelCase z odrzuceniami `reason: {kind}` (`fuel/commands.rs:95-157`), zapis przez kopię danych i `UnlockedVault::commit` z wycofaniem w pamięci (`fuel/commands.rs:348-391`, `vault/mod.rs:161-170`).
- Komendy paliwowe są `async`, ale wykonują blokujący zapis pliku na wątku runtime'u — uwaga F4 z recenzji S-02, świadomie nienaprawiona (`context/archive/2026-10-11-first-fillup-consumption/reviews/impl-review.md:56-64`). Komendy sejfu robią IO w `spawn_blocking` (`vault/commands.rs:117-128`).
- Frontend: `AppShell` po punkcie startowym pokazuje jedną stronę — karty paliw, `FillUpForm`, dwie historie (`src/AppShell.tsx:64-85`); bez punktu startowego tylko `StartPointScreen` (`AppShell.tsx:66-68`). Brak nawigacji/zakładek.
- Brak runnera testów frontendu; bramką jest `npm run build` (tsc strict, `noUnusedLocals`).

## Desired End State

- Po odblokowaniu i z ustawionym punktem startowym pod nagłówkiem są dwie zakładki: „Paliwo” (domyślna, dotychczasowa zawartość bez zmian) i „Serwisy”.
- Zakładka „Serwisy” ma formularz (dodawanie/edycja) i listę „Rejestr serwisów”. Wpis ma: nazwę (wymagana), notatkę (opcjonalna), próg przebiegu i/lub termin (co najmniej jeden), znacznik „Cykliczny” z interwałem „co N km” dla progu przebiegu i „co M miesięcy” dla terminu.
- Lista: najpierw wpisy z terminem rosnąco po dacie, potem wpisy z samym progiem przebiegu rosnąco po km; remis rozstrzyga `id`. Pusta lista: „Brak zaplanowanych serwisów”.
- „Edytuj” wczytuje wpis do formularza; zapis zmienia wpis w miejscu (to samo `id`). „Usuń” zamienia wiersz w „Usunąć serwis „<nazwa>”? [Usuń] [Anuluj]”; dopiero drugie „Usuń” usuwa (wzorzec wspólny z S-04).
- Progi przyjmowane tak, jak wpisane — bez porównania z przebiegiem auta czy dzisiejszą datą.
- Dane po restarcie i odblokowaniu identyczne; sejf jest w formacie v3; pierwszy zapis po aktualizacji zostawił `kmplus.vault.pre-v3-<s>` identyczny bajtowo z plikiem sprzed aktualizacji.
- Wydanie własne: 0.1.7, jeśli S-05 scali się przed S-04, w przeciwnym razie kolejny numer po wydaniu S-04.

### Key Discoveries:

- `src-tauri/src/vault/mod.rs:134-141` — nazwa kopii przed migracją wynika z `data::CURRENT_FORMAT_VERSION`; podbicie do 3 daje `pre-v3-*` bez zmian w `save()`.
- `src-tauri/src/vault/data.rs:49-50` — klucz wymagany, ale wartość nullable wymaga `#[serde(deserialize_with = "Option::deserialize")]`; bez tego serde potraktuje brak pola `Option` jako `None`, łamiąc regułę „brak pola = uszkodzenie”.
- `src-tauri/src/fuel/mod.rs:19,330` — `MAX_ODOMETER_KM` i `is_valid_date` są publiczne; walidacja serwisu używa ich zamiast kopii.
- Tauri 2: komendy bez `async` działają na wątku głównym (zamrażają UI), `async` na zadaniu runtime'u (dokumentacja Tauri v2 „Calling Rust”, Context7 `/tauri-apps/tauri-docs`). Stąd dla zapisu: `async` + `spawn_blocking`, nie zwykłe `fn` (sugestia F4 „plain sync fn” byłaby tu gorsza).
- `src/AppShell.tsx:76-77` — formularz tankowania ma zostać zamontowany przy odświeżeniu („its fuel, date and result survive”); przełączanie zakładek nie może tego zepsuć.

## What We're NOT Doing

- Tworzenie kolejnego wystąpienia dla wpisu cyklicznego i arytmetyka „+ M miesięcy” (S-08).
- Ocena, czy próg został osiągnięty, przypomnienia, status „wymaga uwagi”, sortowanie po pilności (S-07).
- Oznaczanie serwisu jako wykonany, historia wykonanych serwisów (S-08).
- Porównywanie progu z najnowszym przebiegiem lub dzisiejszą datą — ani odrzucenie, ani ostrzeżenie (decyzja: przyjmować tak, jak wpisane).
- Pole rodzaju wpisu (przegląd/wymiana) — decyzja: tylko nazwa + notatka.
- Cofanie usunięcia, kosz, eksport — decyzja: dwuetapowe potwierdzenie w wierszu.
- Rejestr dostępny przed ustawieniem punktu startowego — zakładki są za istniejącą bramką `StartPointScreen`.
- Zapamiętywanie wybranej zakładki między uruchomieniami; router.
- Samodzielny wpis przebiegu (S-06); zmiany w logice paliw, `fuel_overview` i komponentach paliwowych (obszar S-04).
- Naprawa F4 w istniejących komendach paliwowych — tylko nowe komendy jej nie powtarzają.

## Implementation Approach

Kolejność jak w S-02/S-03: czysta domena w Ruście → format sejfu → cienkie komendy Tauri z kontraktem → UI wyłącznie wyświetlające → wydanie z dowodem migracji. Serwisy dostają własny moduł `service` (lustro `fuel`) i własne komendy, żeby nie dotykać `fuel_overview` ani plików paliwowych, które zmienia równoległy S-04. Model przechowuje próg absolutny (km na liczniku, data) i opcjonalny interwał dla każdego zadanego progu — dzięki temu S-08 policzy kolejne wystąpienie od bieżącego stanu bez dodatkowej migracji. Kolejność listy liczy backend; frontend jej nie zmienia.

Wartości graniczne ustalone przez agenta (niezakwestionowane przez użytkownika): interwał km `1..=9 999 999`, interwał miesięcy `1..=120`, nazwa 1–100 znaków po przycięciu, notatka do 500 znaków po przycięciu, pusta notatka zapisywana jako brak (`null`).

## Critical Implementation Details

- **Zgodność starszej aplikacji:** po pierwszym zapisie w v3 aplikacje 0.1.6 i starsze odmówią otwarcia sejfu (`tooNew`) — tak ma być. Powrót do starszej wersji wymaga ręcznego przywrócenia `pre-v3-*`; nie budujemy ścieżki wstecz.
- **Zakładki a stan formularza:** obie zakładki pozostają zamontowane, nieaktywna ukryta (np. atrybut `hidden`), żeby przełączenie nie kasowało stanu `FillUpForm` ani formularza serwisu.
- **Style w osobnym pliku:** wszystkie nowe style S-05 (pasek zakładek, formularz, rejestr) trafiają do `src/service/service.css`, importowanego w `src/service/ServicesTab.tsx`, z własnym blokiem `@media (prefers-color-scheme: dark)` w tych samych kolorach co `App.css`; `src/App.css` pozostaje bez zmian (zmienia go S-04).
- **Koordynacja z równoległym S-04 (`correct-or-delete-fillup`):** ręczna weryfikacja Fazy 3 i cała Faza 4 nie mogą nakładać się w czasie z ręczną weryfikacją Fazy 3 ani z Fazą 4 S-04. Obie zmiany używają tego samego katalogu danych (`%APPDATA%\io.github.m-swietek.kmplus`) i portu 1420 (`strictPort`), a build deweloperski S-05 zapisuje sejf w formacie v3 (zainstalowana 0.1.6 odrzuci go jako `tooNew`). Przed odłożeniem prawdziwych plików upewnij się, że S-04 nie ma właśnie odłożonego prawdziwego sejfu ani działającego `tauri dev`. Wydania idą po kolei: zalecana kolejność to S-04 scalone i wydane pierwsze, potem S-05 robi rebase na `main` (owinięcie bloku paliwowego w zakładkę na gotowym JSX z S-04), powtarza testy automatyczne i bierze kolejny numer wersji.
- **Blok „Paliwo” przenoszony w całości:** karty, `FillUpForm` i `.fuel-histories` zostają tym samym JSX w `AppShell`, tylko owinięte w kontener zakładki — bez wydzielania do nowego komponentu i bez zmian w `FillUpForm`/`FillUpHistory`, żeby zmiany S-04 w tych komponentach scaliły się czysto.

## Phase 1: Domena serwisu i format sejfu v3

### Overview

Czysty moduł `service` z typami, walidacją i kolejnością rejestru; ładunek sejfu v3 z migracją v2 → v3 (i łańcuchem v1 → v2 → v3); testy migracji i kopii `pre-v3`; rejestr kontraktu dla ładunku.

### Changes Required:

#### 1. Moduł domeny serwisu

**File**: `src-tauri/src/service/mod.rs` (nowy), `src-tauri/src/lib.rs` (`mod service;`)

**Intent**: Typy zapisywane w sejfie, walidacja nowego/edytowanego wpisu i deterministyczna kolejność rejestru — bez Tauri i bez IO, jak `fuel/mod.rs`.

**Contract**:
- `PlannedService { id: u64, name: String, note: Option<String>, due_odometer_km: Option<u32>, due_date: Option<String>, recurrence: Option<Recurrence> }` — serde, snake_case; każde pole `Option` z `deserialize_with = "Option::deserialize"` (klucz wymagany, wartość może być `null`).
- `Recurrence { every_km: Option<u32>, every_months: Option<u32> }` — ta sama zasada kluczy wymaganych.
- `NewService` (bez `id`) + `with_id(id)`; `normalize`: `name` i `note` przycięte, pusta notatka → `None`.
- `validate_service(&NewService) -> Result<(), ServiceRejection>` (po normalizacji), kolejność sprawdzeń: `NameEmpty`, `NameTooLong` (> 100 znaków Unicode), `NoteTooLong` (> 500), `NoThreshold` (oba progi `None`), `OdometerOutOfRange` (`1..=MAX_ODOMETER_KM`), `InvalidDate` (`fuel::is_valid_date`), dla `recurrence: Some`: `IntervalMismatch` (interwał km jest ⇔ próg przebiegu jest; interwał miesięcy jest ⇔ termin jest), `IntervalKmOutOfRange` (`1..=MAX_ODOMETER_KM`), `IntervalMonthsOutOfRange` (`1..=120`).
- Bez porównania progów z punktem startowym, tankowaniami ani dzisiejszą datą.
- `register_order(&[PlannedService]) -> Vec<&PlannedService>` (lub równoważne): klucz sortowania — wpisy z `due_date` przed wpisami bez; wśród z datą: `(due_date, id)`; wśród bez daty: `(due_odometer_km, id)`. Wpis z oboma progami sortuje się po dacie.

#### 2. Testy domeny

**File**: `src-tauri/src/service/tests.rs` (nowy)

**Intent**: Udowodnić każdą regułę walidacji i kolejność rejestru.

**Contract**: testy co najmniej:
- każdy wariant `ServiceRejection` na minimalnym przypadku, w tym granice (`name` 100 vs 101 znaków z polskimi znakami, `note` 500 vs 501, `every_months` 120 vs 121, `every_km` 0, próg km 0 i 10 000 000);
- przycięcie nazwy i notatki; notatka z samych spacji → `None`;
- tylko próg km, tylko termin, oba — przyjęte; próg w przeszłości (termin 2020-01-01, km 1) przyjęty;
- `IntervalMismatch`: cykliczny z progiem km bez `every_km`; cykliczny z `every_months` bez terminu; cykliczny bez żadnego interwału;
- kolejność: A termin 2027-03-01, B 115 000 km, C 180 000 km + 2028-06-01, D termin 2026-10-01, E 115 000 km (wyższe id niż B) → `[D, A, C, B, E]`.

#### 3. Ładunek v3 i migracja

**File**: `src-tauri/src/vault/data.rs`

**Intent**: Dodać serwisy do sejfu zgodnie z regułą „nowa wersja + migrator”, zachowując istniejące dane bez zmian.

**Contract**:
- `CURRENT_FORMAT_VERSION = 3`; `VaultData` dostaje `services: Vec<PlannedService>` i `next_service_id: u64` (start `1`, id nigdy nie są używane ponownie); `VaultData::new` odpowiednio.
- Zamrożona kopia `VaultDataV2` (dotychczasowe pola) do dekodowania v2; `migrate_v2_to_v3(v2) -> VaultData` przenosi `created_at`, `start_point`, `fill_ups`, `next_fill_up_id` bez zmian, `services: []`, `next_service_id: 1`.
- `decode`: `1` → v1 → v2 → v3 (`migrated_from: Some(1)`); `2` → v3 (`migrated_from: Some(2)`); `3` → bez migracji; `> 3` → `TooNew`; inne → `Malformed`. Migratory to osobne funkcje wywoływane łańcuchowo.
- Komentarze modułu (`data.rs:1-5`, `vault/mod.rs:13-15`) zaktualizowane do v3.

#### 4. Testy sejfu

**File**: `src-tauri/src/vault/tests.rs`

**Intent**: Przenieść istniejące testy formatu na v3 i dodać dowód migracji v2 → v3 na plikach.

**Contract**:
- `newer_format_version_is_too_new` używa `format_version: 4`.
- `v1_payload_migrates_to_empty_v2` → migruje do pustego v3 (nazwa testu zaktualizowana); round-trip bieżącego ładunku z serwisem.
- Nowy `v2_payload_migrates_to_v3_keeping_fill_ups`: ładunek v2 z punktem startowym i tankowaniami → identyczne pola paliwowe, `services: []`, `next_service_id: 1`, `migrated_from: Some(2)`.
- `v3_missing_field_is_malformed`: brak każdego pola najwyższego poziomu (w tym `services`, `next_service_id`) i brak klucza w serwisie (np. `note`, `recurrence`) oraz w `Recurrence` → `Malformed`; v2 z brakującym polem nadal `Malformed`.
- Nowe testy kopii dla v2: `unlock_of_v2_vault_writes_nothing`, `first_save_after_v2_migration_writes_pre_v3_copy_of_v2_bytes`; istniejące testy v1 (`first_save_after_migration_…`, `second_save_writes_no_further_copy`, `restore_backup_from_v1_then_save_…`, `v2_vault_save_writes_no_pre_migration_copy` → v3) przechodzą z nazwą kopii `pre-v3-*`.

#### 5. Rejestr kontraktu — ładunek

**File**: `docs/reference/contract-surfaces.md`

**Intent**: Wpisać format v3 i pola serwisów jako nazwy nośne.

**Contract**: sekcja „Vault format” (`format_version` = `3`, migracja v1 → v2 → v3), „Vault files” (przykład `pre-v3-*`), „Vault payload” — nagłówek „JSON v3” i wiersze: `services`, `next_service_id`, `PlannedService.id|name|note|due_odometer_km|due_date|recurrence`, `Recurrence.every_km|every_months` z jednostkami i zakresami.

### Success Criteria:

#### Automated Verification:

- Testy domeny serwisu przechodzą: `cargo test service::tests` w `src-tauri/`
- Testy sejfu (migracja v2 → v3, kopia `pre-v3`, brakujące pola) przechodzą: `cargo test vault::tests` w `src-tauri/`
- Cały zestaw testów przechodzi: `cargo test` w `src-tauri/`
- Brak ostrzeżeń kompilacji (w tym martwego kodu): `cargo check` w `src-tauri/`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase. (`cargo` spoza PATH: `$HOME\.cargo\bin\cargo`.)

---

## Phase 2: Komendy i kontrakt serwisów

### Overview

Cztery komendy Tauri nad domeną serwisu, DTO i testy kształtu JSON, rejestracja w `generate_handler!`, rejestr komend i typy TS.

### Changes Required:

#### 1. DTO i komendy

**File**: `src-tauri/src/service/commands.rs` (nowy)

**Intent**: Cienka warstwa jak `fuel/commands.rs`: odczyt rejestru i trzy zapisy przez `UnlockedVault::commit`, z IO poza wątkiem runtime'u.

**Contract** (camelCase, unie z `kind`):
- `service_register` → `ServiceRegister = { services: ServiceView[] }` w kolejności `register_order`; `ServiceView = { id, name, note: string | null, dueOdometerKm: number | null, dueDate: string | null, recurrence: { everyKm: number | null, everyMonths: number | null } | null }`.
- `add_service(input: ServiceInput)` → `AddServiceResult`: `saved` (+ `id`), `rejected` (+ `reason: ServiceRejection`).
- `update_service(id, input: ServiceInput)` → `UpdateServiceResult`: `saved`, `notFound`, `rejected` (+ `reason`). Zastępuje wszystkie pola wpisu o tym `id`, `id` bez zmian; walidacja jak przy dodawaniu.
- `delete_service(id)` → `DeleteServiceResult`: `deleted`, `notFound`.
- `ServiceInput = { name, note: string | null, dueOdometerKm: number | null, dueDate: string | null, recurrence: { everyKm, everyMonths } | null }`.
- `ServiceRejection` (`{ kind }`): `nameEmpty`, `nameTooLong`, `noteTooLong`, `noThreshold`, `odometerOutOfRange`, `invalidDate`, `intervalMismatch`, `intervalKmOutOfRange`, `intervalMonthsOutOfRange`.
- Komendy nie wymagają punktu startowego (bramka jest tylko w UI). Odrzucenie i `notFound` niczego nie zapisują. Zablokowana sesja → `Err("dane są zablokowane; odblokuj aplikację")`.
- Każda komenda `async`, przyjmuje `AppHandle`; praca na sesji (blokada mutexa, `commit`) w `tauri::async_runtime::spawn_blocking`, gdzie sesja pobierana jest przez `app.state::<VaultSession>()` — `State<'_, _>` nie jest `'static`, więc nie może wejść do zamknięcia.
- Logika zapisu w funkcjach `apply_*` na `&mut UnlockedVault` (testowalnych bez Tauri), jak `apply_fill_up`.

#### 2. Rejestracja

**File**: `src-tauri/src/lib.rs`

**Intent**: Dodać cztery komendy do `generate_handler!`, inaczej wywołanie z frontendu zawodzi w czasie działania.

**Contract**: `service_register`, `add_service`, `update_service`, `delete_service` dopisane na końcu listy (mniejsza kolizja z S-04).

#### 3. Testy komend

**File**: `src-tauri/src/service/commands.rs` (`mod tests`)

**Intent**: Zamrozić kształt JSON i zachowanie zapisu na prawdziwym sejfie.

**Contract**: testy kształtu JSON dla każdego wariantu wyników i odrzuceń oraz deserializacji `ServiceInput` (w tym `null` w polach i `recurrence: null`); integracyjne na `TempDir` z tanim KDF (wzorzec `fuel/commands.rs` testy): dodanie nadaje rosnące id od 1 i zapisuje (ponowne odblokowanie widzi wpis), edycja zachowuje id i zmienia pola, usunięcie usuwa, id po usunięciu nie jest używane ponownie, `notFound` dla nieistniejącego id przy edycji i usuwaniu bez zmiany pliku, odrzucenie bez zmiany pliku, rejestr zwraca kolejność z Fazy 1.

#### 4. Rejestr kontraktu — komendy

**File**: `docs/reference/contract-surfaces.md`

**Intent**: Dopisać cztery komendy z argumentami i wartościami `kind`, oraz `src-tauri/src/service/commands.rs` jako miejsce definicji.

**Contract**: wiersze tabeli „Tauri commands” + lista typów zagnieżdżonych (`ServiceView`, `ServiceInput`, `ServiceRejection`).

#### 5. Typy frontendu

**File**: `src/service/api.ts` (nowy)

**Intent**: Odbicie DTO 1:1 i wrappery `invoke`: `serviceRegister()`, `addService(input)`, `updateService(id, input)`, `deleteService(id)`.

**Contract**: typy zgodne z DTO; komentarz nagłówkowy wskazujący `contract-surfaces.md`, jak `src/fuel/api.ts:1-3`.

### Success Criteria:

#### Automated Verification:

- Testy komend i kształtu JSON przechodzą: `cargo test service::commands` w `src-tauri/`
- Cały zestaw testów przechodzi: `cargo test` w `src-tauri/`
- Brak ostrzeżeń kompilacji: `cargo check` w `src-tauri/`
- Frontend kompiluje się z nowymi typami: `npm run build`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 3: UI rejestru serwisów i zakładki

### Overview

Zakładki „Paliwo” / „Serwisy” w `AppShell`, formularz dodawania/edycji z warunkowymi interwałami, lista rejestru z „Edytuj” i dwuetapowym „Usuń”, komunikaty po polsku, style z trybem ciemnym.

### Changes Required:

#### 1. Zakładki

**File**: `src/AppShell.tsx`, `src/service/service.css` (nowy)

**Intent**: Pasek dwóch zakładek pod `h1`, stan lokalny `"fuel" | "services"`, domyślnie „Paliwo”; blok paliwowy bez zmian w treści.

**Contract**: przyciski zakładek z `aria-pressed` (lub `role="tab"`/`aria-selected`); obie zawartości zamontowane, nieaktywna ukryta (patrz Critical Implementation Details); zawartość „Serwisy” = `<ServicesTab />`.

#### 2. Zakładka serwisów

**File**: `src/service/ServicesTab.tsx` (nowy)

**Intent**: Właściciel stanu rejestru: wczytanie `serviceRegister()` przy montowaniu (stan wczytywania/błędu z „Spróbuj ponownie” jak `AppShell`), id edytowanego wpisu, odświeżenie po każdym zapisie i usunięciu.

**Contract**: po usunięciu wpisu, który jest właśnie edytowany, formularz wraca do trybu dodawania; `notFound` przy edycji/usuwaniu → komunikat „Ten wpis już nie istnieje.” i odświeżenie listy.

#### 3. Formularz

**File**: `src/service/ServiceForm.tsx` (nowy)

**Intent**: Jeden formularz dla dodawania i edycji (wzorzec `FillUpForm`): Nazwa, Notatka (opcjonalna), Próg przebiegu (km, opcjonalny), Termin (`<input type="date">`, opcjonalny), „Cykliczny”; po zaznaczeniu pola „Co ile km” (gdy podano próg przebiegu) i „Co ile miesięcy” (gdy podano termin).

**Contract**:
- Parsowanie `parseKm` z `src/fuel/format.ts`; liczby całkowite miesięcy; sprawdzenie `U32_MAX` przed `invoke` jak w `FillUpForm.tsx:68-76`.
- Odznaczenie „Cykliczny” wysyła `recurrence: null`; interwały pól niewidocznych nie są wysyłane (pole progu wyczyszczone → jego interwał też).
- Przyciski: „Zapisz serwis” (dodawanie) / „Zapisz zmiany” + „Anuluj” (edycja); dane formularza zostają przy odrzuceniu i `Err`; po zapisie formularz czyści się do trybu dodawania.
- Komunikat po polsku dla każdego `kind` odrzucenia, m.in. `noThreshold` → „Podaj próg przebiegu, termin albo oba.”, `intervalMismatch` → „Dla wpisu cyklicznego podaj interwał dla każdego zadanego progu.”
- Bez ostrzeżeń o progu w przeszłości.

#### 4. Lista rejestru

**File**: `src/service/ServiceRegister.tsx` (nowy), `src/service/service.css`

**Intent**: Wiersze w kolejności z backendu: nazwa; progi („przy 115 000 km”, „do 01.03.2027”, przy obu „przy … lub do …”); cykliczność („co 15 000 km”, „co 12 mies.”); notatka; przyciski „Edytuj” i „Usuń”.

**Contract**: „Usuń” w wierszu przechodzi w „Usunąć serwis „<nazwa>”? [Usuń] [Anuluj]” (wzorzec wspólny z S-04: „Usunąć <opis wpisu>? [Usuń] [Anuluj]”); „Anuluj” wraca do stanu wyjściowego, potwierdzające „Usuń” wywołuje `deleteService`; w danym momencie potwierdzenie otwarte w najwyżej jednym wierszu. Pusta lista: „Brak zaplanowanych serwisów”. Nagłówek „Rejestr serwisów”. Formatery `formatKm`, `formatDate`. Style zgodne z `.fuel-history-*`, czytelne w trybie ciemnym i przy wąskim oknie.

### Success Criteria:

#### Automated Verification:

- Frontend kompiluje się bez nieużywanych importów: `npm run build`
- Testy backendu nadal przechodzą: `cargo test` w `src-tauri/`

#### Manual Verification:

- `npm run tauri dev` na czystym katalogu danych (prawdziwy `kmplus.vault` i pliki obok przeniesione na bok przed testem i przywrócone po nim, z porównaniem SHA-256): po haśle i punkcie startowym (2026-01-01, 100 000 km) widać zakładkę „Paliwo” jak dotąd; „Serwisy” pokazuje „Brak zaplanowanych serwisów”
- Dodanie „Przegląd techniczny” (termin 2027-03-01), „Wymiana oleju” (115 000 km, cykliczny co 15 000 km, notatka „5W-30”), „Rozrząd” (180 000 km i 2028-06-01, cykliczny co 90 000 km i co 48 mies.) oraz „Płyn hamulcowy” (termin 2026-10-01, w przeszłości — zapisany bez ostrzeżenia): kolejność listy Płyn hamulcowy, Przegląd techniczny, Rozrząd, Wymiana oleju; progi, cykliczność i notatka widoczne
- Zapis bez progu → „Podaj próg przebiegu, termin albo oba.”; cykliczny z terminem bez liczby miesięcy → komunikat o interwale; dane formularza zostają
- „Edytuj” na „Wymiana oleju”: próg 116 000 km, odznaczony „Cykliczny” → wiersz bez cykliczności, ten sam wpis (nie nowy); „Anuluj” w edycji nie zmienia wpisu
- „Usuń” przy „Płyn hamulcowy” pokazuje „Usunąć serwis „Płyn hamulcowy”?”; „Anuluj” zostawia wpis; potwierdzające „Usuń” usuwa „Płyn hamulcowy”
- Przełączanie zakładek nie kasuje wpisanych pól formularza tankowania ani serwisu; dodanie tankowania nadal działa
- Po restarcie i odblokowaniu rejestr identyczny; tryb ciemny i wąskie okno czytelne

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 4: Wydanie i dowód migracji v2 → v3

### Overview

Własne wydanie istniejącym pipeline'em według rytuału z S-02/S-03 i `context/deployment/deploy-plan.md`: workflow startuje tylko po wyraźnym „start” użytkownika, draft publikuje wyłącznie użytkownik. Dowodem jest aktualizacja z zainstalowanej wersji v2 na prawdziwym sejfie: odblokowanie nic nie pisze, pierwszy zapis tworzy `pre-v3-*` identyczny z plikiem sprzed aktualizacji.

### Changes Required:

#### 1. Wersja

**File**: `src-tauri/tauri.conf.json`, `package.json`, `src-tauri/Cargo.toml` (+ `Cargo.lock`, `package-lock.json`)

**Intent**: Podbić wersję do następnej wolnej: 0.1.7, jeśli S-04 nie został jeszcze wydany; w przeciwnym razie kolejny numer po wydaniu S-04 (np. 0.1.8). Numer ustalany w chwili tej fazy z `gh release list`.

**Contract**: wersje zgodne w trzech miejscach; tag `app-v<X>`; commit, push na `main`, `gh workflow run release.yml --ref main` po „start”.

#### 2. Zapis przebiegu

**File**: `context/changes/plan-service/change.md` (`## Notes`), `context/deployment/deploy-plan.md` (`## Przebieg wykonania`)

**Intent**: Ślad dowodowy z wartościami (nie samo potwierdzenie słowne — F1 z recenzji S-02).

**Contract**: numer runu z zielonym krokiem „Rust tests”; rozmiar i SHA-256 `kmplus.vault` przed aktualizacją; SHA-256 po odblokowaniu bez zapisu; nazwa i SHA-256 pliku `pre-v3-*`; zainstalowana wersja.

### Success Criteria:

#### Automated Verification:

- Workflow wydania kończy się sukcesem, krok „Rust tests” zielony: `gh run view <id>` na runie `release.yml`
- Draft `app-v<X>` zawiera instalator, `.sig` i `latest.json`: `gh release view app-v<X>`

#### Manual Verification:

- Przed aktualizacją: w `%APPDATA%\io.github.m-swietek.kmplus\` leży prawdziwy sejf (nie testowy z Fazy 3 — potwierdzenie użytkownika i porównanie z ostatnim zapisanym dowodem); zanotowane rozmiar i SHA-256 `kmplus.vault` na zainstalowanej poprzedniej wersji
- Aktualizacja przez baner; to samo hasło odblokowuje; SHA-256 `kmplus.vault` po odblokowaniu bez zapisu identyczny jak przed aktualizacją; brak nowego pliku `pre-v3-*`; tankowania i karty paliw jak przed aktualizacją; „Serwisy” pusta
- Pierwsze dodanie serwisu tworzy `kmplus.vault.pre-v3-<s>` o SHA-256 równym stanowi sprzed aktualizacji; po restarcie i odblokowaniu serwis i tankowania widoczne; przebieg zapisany w `change.md` i `deploy-plan.md`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Testing Strategy

### Unit Tests:

- `service::tests`: każdy wariant `ServiceRejection` z granicami (100/101 znaków nazwy, 500/501 notatki, 120/121 miesięcy, km 0 i 10 000 000), przycinanie i pusta notatka, progi w przeszłości przyjęte, trzy przypadki `IntervalMismatch`, kolejność `[D, A, C, B, E]`.
- `vault::tests`: `v2_payload_migrates_to_v3_keeping_fill_ups`, migracja v1 → v3, `v3_missing_field_is_malformed`, `newer_format_version_is_too_new` dla v4, kopie `pre-v3` dla źródła v1 i v2, brak kopii przy zapisie v3.

### Integration Tests:

- `service::commands` na sejfie w `TempDir` z tanim KDF: add/update/delete z ponownym odblokowaniem, id rosnące i nieużywane ponownie, `notFound` i odrzucenia bez zmiany pliku, kolejność rejestru; testy kształtu JSON każdego DTO.

### Manual Testing Steps:

1. Scenariusz z Fazy 3 krok po kroku na czystym katalogu danych (prawdziwy sejf odłożony na bok i przywrócony z kontrolą SHA-256).
2. Restart i odblokowanie — rejestr i paliwa bez zmian.
3. Aktualizacja z zainstalowanej wersji v2 na prawdziwych danych (Faza 4).

## Performance Considerations

Rejestr jednego auta to kilkanaście wpisów; sortowanie przy każdym odczycie bez znaczenia. Zapis jak dotąd: pełny sejf przez `commit`, poza wątkiem runtime'u.

## Migration Notes

- Format v2 → v3: migracja w pamięci przy odblokowaniu (nic nie pisze), zapis na dysk dopiero przy pierwszej zmianie danych, poprzedzony kopią `kmplus.vault.pre-v3-<s>` (nigdy nie usuwaną) — istniejący mechanizm `save()`.
- Sejfy v1 (sprzed 0.1.4) migrują łańcuchowo v1 → v2 → v3.
- Wycofanie: starsze wersje odrzucą v3 jako `tooNew`; powrót tylko przez ręczne przywrócenie `pre-v3-*` przy starszej aplikacji.

## Open Risks & Assumptions

- **Kolizje scalenia z S-04** (osobny worktree, ten sam commit bazowy): `src-tauri/src/lib.rs` (`generate_handler!` — obie zmiany dopisują komendy; dopisujemy na końcu listy), tabela komend w `docs/reference/contract-surfaces.md`, `src/AppShell.tsx` (owinięcie bloku paliwowego w zakładkę — konflikt pewny, rozwiązanie: struktura zakładek z S-05 + propsy i hook z S-04), podbicie wersji (obie zmiany celują w 0.1.7 — wygrywa ta scalona pierwsza, druga bierze kolejny numer). S-04 nie zmienia formatu sejfu, więc `vault/data.rs` i numer `format_version` nie kolidują; gdyby S-04 jednak dodał pole, ta zmiana musiałaby zostać v4 i migratorem v3 → v4 po S-05 (albo odwrotnie). `src/App.css` nie koliduje — S-05 ma własny `src/service/service.css`. Kolejność faz ręcznych i wydań — patrz Critical Implementation Details.
- **Arytmetyka miesięcy** (31.01 + 1 mies.) świadomie odłożona do S-08; S-05 zapisuje tylko liczbę miesięcy.
- **Kolejność rejestru** jest bazowa; S-07 może wyróżnić wpisy wymagające uwagi bez zmiany tej reguły.
- **Testowy sejf w prawdziwym katalogu** (F2 z recenzji S-03): przed Fazą 4 obowiązkowa kontrola, że leży prawdziwy sejf.

## References

- Roadmap: `context/foundation/roadmap.md` (S-05)
- PRD: `context/foundation/prd.md` (US-02, FR-008, FR-009; FR-011 — podstawa interwału)
- Poprzednie zmiany: `context/archive/2026-10-11-first-fillup-consumption/plan.md` (migracja v1 → v2, rytuał wydania), `context/archive/2026-10-26-fillup-history-both-fuels/plan.md`
- Format i migracja: `src-tauri/src/vault/data.rs:38-128`, `src-tauri/src/vault/mod.rs:118-170`
- Wzorzec domeny/komend: `src-tauri/src/fuel/mod.rs:113-178`, `src-tauri/src/fuel/commands.rs:95-391`
- IO poza runtime'em: `src-tauri/src/vault/commands.rs:117-128`
- Powłoka UI: `src/AppShell.tsx:64-85`, formularz: `src/fuel/FillUpForm.tsx`
- Dokumentacja Tauri v2 „Calling Rust” (Async Commands) — Context7 `/tauri-apps/tauri-docs`

## Progress

> Convention: `- [ ]` pending, `- [x]` done. Append ` — <commit sha>` when a step lands. Do not rename step titles. See `references/progress-format.md`.

### Phase 1: Domena serwisu i format sejfu v3

#### Automated

- [x] 1.1 Testy domeny serwisu przechodzą: `cargo test service::tests` w `src-tauri/` — c475eca
- [x] 1.2 Testy sejfu (migracja v2 → v3, kopia `pre-v3`, brakujące pola) przechodzą: `cargo test vault::tests` w `src-tauri/` — c475eca
- [x] 1.3 Cały zestaw testów przechodzi: `cargo test` w `src-tauri/` — c475eca
- [x] 1.4 Brak ostrzeżeń kompilacji (w tym martwego kodu): `cargo check` w `src-tauri/` — c475eca

### Phase 2: Komendy i kontrakt serwisów

#### Automated

- [x] 2.1 Testy komend i kształtu JSON przechodzą: `cargo test service::commands` w `src-tauri/` — 40b52e2
- [x] 2.2 Cały zestaw testów przechodzi: `cargo test` w `src-tauri/` — 40b52e2
- [x] 2.3 Brak ostrzeżeń kompilacji: `cargo check` w `src-tauri/` — 40b52e2
- [x] 2.4 Frontend kompiluje się z nowymi typami: `npm run build` — 40b52e2

### Phase 3: UI rejestru serwisów i zakładki

#### Automated

- [x] 3.1 Frontend kompiluje się bez nieużywanych importów: `npm run build` — b4aedb2
- [x] 3.2 Testy backendu nadal przechodzą: `cargo test` w `src-tauri/` — b4aedb2

#### Manual

- [x] 3.3 `npm run tauri dev` na czystym katalogu danych (prawdziwy `kmplus.vault` i pliki obok przeniesione na bok przed testem i przywrócone po nim, z porównaniem SHA-256): po haśle i punkcie startowym (2026-01-01, 100 000 km) widać zakładkę „Paliwo” jak dotąd; „Serwisy” pokazuje „Brak zaplanowanych serwisów” — a17de95
- [x] 3.4 Dodanie „Przegląd techniczny” (termin 2027-03-01), „Wymiana oleju” (115 000 km, cykliczny co 15 000 km, notatka „5W-30”), „Rozrząd” (180 000 km i 2028-06-01, cykliczny co 90 000 km i co 48 mies.) oraz „Płyn hamulcowy” (termin 2026-10-01, w przeszłości — zapisany bez ostrzeżenia): kolejność listy Płyn hamulcowy, Przegląd techniczny, Rozrząd, Wymiana oleju; progi, cykliczność i notatka widoczne — a17de95
- [x] 3.5 Zapis bez progu → „Podaj próg przebiegu, termin albo oba.”; cykliczny z terminem bez liczby miesięcy → komunikat o interwale; dane formularza zostają — a17de95
- [x] 3.6 „Edytuj” na „Wymiana oleju”: próg 116 000 km, odznaczony „Cykliczny” → wiersz bez cykliczności, ten sam wpis (nie nowy); „Anuluj” w edycji nie zmienia wpisu — a17de95
- [x] 3.7 „Usuń” przy „Płyn hamulcowy” pokazuje „Usunąć serwis „Płyn hamulcowy”?”; „Anuluj” zostawia wpis; potwierdzające „Usuń” usuwa „Płyn hamulcowy” — a17de95
- [x] 3.8 Przełączanie zakładek nie kasuje wpisanych pól formularza tankowania ani serwisu; dodanie tankowania nadal działa — a17de95
- [x] 3.9 Po restarcie i odblokowaniu rejestr identyczny; tryb ciemny i wąskie okno czytelne — a17de95

### Phase 4: Wydanie i dowód migracji v2 → v3

#### Automated

- [ ] 4.1 Workflow wydania kończy się sukcesem, krok „Rust tests” zielony: `gh run view <id>` na runie `release.yml`
- [ ] 4.2 Draft `app-v<X>` zawiera instalator, `.sig` i `latest.json`: `gh release view app-v<X>`

#### Manual

- [ ] 4.3 Przed aktualizacją: w `%APPDATA%\io.github.m-swietek.kmplus\` leży prawdziwy sejf (nie testowy z Fazy 3 — potwierdzenie użytkownika i porównanie z ostatnim zapisanym dowodem); zanotowane rozmiar i SHA-256 `kmplus.vault` na zainstalowanej poprzedniej wersji
- [ ] 4.4 Aktualizacja przez baner; to samo hasło odblokowuje; SHA-256 `kmplus.vault` po odblokowaniu bez zapisu identyczny jak przed aktualizacją; brak nowego pliku `pre-v3-*`; tankowania i karty paliw jak przed aktualizacją; „Serwisy” pusta
- [ ] 4.5 Pierwsze dodanie serwisu tworzy `kmplus.vault.pre-v3-<s>` o SHA-256 równym stanowi sprzed aktualizacji; po restarcie i odblokowaniu serwis i tankowania widoczne; przebieg zapisany w `change.md` i `deploy-plan.md`
