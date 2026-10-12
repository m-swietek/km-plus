# Pierwsze tankowanie daje spalanie (S-02) — Implementation Plan

## Overview

Gwiazda przewodnia roadmapy: po odblokowaniu sejfu użytkownik jednorazowo podaje punkt startowy (data i przebieg chwili, gdy oba baki były pełne), dodaje wpis tankowania (paliwo, data, przebieg, litry, kwota, czy do pełna) i bez dodatkowej akcji widzi wynik: spalanie w l/100 km i koszt okresu dla tankowania do pełna albo „dolewka — brak wyniku” dla częściowego, zawsze z ceną za litr. Powłoka aplikacji pokazuje najnowsze zamknięte spalanie osobno dla gazu i benzyny. Przy okazji powstaje pierwsza realna zmiana formatu sejfu (v1 → v2) z migratorem i kopią przed migracją, udowodniona na prawdziwej aktualizacji 0.1.3 → 0.1.4.

Źródło: `context/foundation/roadmap.md` S-02 (US-01, FR-002, FR-003, FR-007), PRD §Business Logic, §Guardrails; dług migracyjny z `context/archive/2026-10-09-password-unlocked-vault/plan.md` (Migration Notes); ryzyko migracji z `context/foundation/infrastructure.md` (Risk Register).

## Current State Analysis

- Sejf z S-01 działa i jest wydany (0.1.3). Jawny ładunek to `VaultData { format_version: 1, created_at }` (`src-tauri/src/vault/data.rs:10-15`); `decode` odrzuca `format_version > 1` jako `TooNew` (`data.rs:39-46`), więc starsza aplikacja nie otworzy pliku v2.
- `UnlockedVault::save()` (`src-tauri/src/vault/mod.rs:102-123`) zapisuje atomowo i rotuje poprzednią wersję do `kmplus.vault.bak`; `.bak` jest nadpisywany przy każdym zapisie, więc sam nie przechowa pliku v1 dłużej niż jeden zapis.
- `unlock` jest wyłącznie odczytem (`mod.rs:160`, kontrakt S-01) — migracja nie może więc zapisywać w momencie odblokowania.
- Sesja: `VaultSession(Mutex<Option<UnlockedVault>>)` (`src-tauri/src/vault/commands.rs:17`) z prywatnym helperem `lock_session` (`commands.rs:170`); komendy rejestrowane w `generate_handler!` (`src-tauri/src/lib.rs:12-17`); wzorzec wyników `#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]` z testami kształtu JSON (`commands.rs:183-229`).
- Frontend: `AppShell` to pusta powłoka „kmPlus / Brak wpisów” (`src/AppShell.tsx`), renderowana w `VaultGate` dopiero po odblokowaniu (`src/App.tsx:47-49`); typowane opakowania `invoke` w `src/vault/api.ts`. Brak runnera testów frontendu — jedyna bramka to `npm run build` (`tsc` strict).
- Testy: 24 testy sejfu w `src-tauri/src/vault/tests.rs` na `TempDir` z tanimi parametrami KDF (`tests.rs:15`); `release.yml` uruchamia `cargo test --locked` przed buildem (zmiana `release-runs-tests`), ale zielony przebieg tej bramki na CI nie był jeszcze widziany.
- `cargo` nie jest w PATH (`context/deployment/deploy-plan.md:12`).

## Desired End State

- Po odblokowaniu sejfu bez punktu startowego użytkownik widzi ekran punktu startowego (data, przebieg w km) z informacją, że to ma być chwila, gdy oba baki były pełne; przed zapisem ekran podsumowania „Punkt startowy: <data>, <przebieg> km — czy dane są poprawne?”. Po zapisie punktu nie da się zmienić.
- Z punktem startowym powłoka pokazuje dwie karty — Gaz i Benzyna — z najnowszym zamkniętym spalaniem (l/100 km, 2 miejsca, data i przebieg tankowania) albo „Brak wyniku”, oraz formularz dodania tankowania.
- Zapis tankowania do pełna pokazuje od razu: spalanie l/100 km, dystans okresu, koszt okresu (zł) i cenę za litr; zapis dolewki pokazuje „Dolewka — brak wyniku spalania” i cenę za litr. Karty paliw odświeżają się.
- Wpis niespójny jest odrzucany z komunikatem wskazującym konflikt; nic nie trafia na dysk.
- Dane przeżywają restart: po ponownym odblokowaniu karty pokazują te same wyniki.
- Sejf 0.1.3 (format v1) po aktualizacji do 0.1.4 otwiera się tym samym hasłem; samo odblokowanie nie zmienia pliku; pierwszy zapis tworzy `kmplus.vault.pre-v2-<unix s>` z nietkniętą kopią pliku v1.

Weryfikacja: `cargo test` (lista w Testing Strategy, w tym test wyroczni), `npm run build`, scenariusze ręczne Fazy 3 i dowód aktualizacji w Fazie 4.

### Key Discoveries:

- `src-tauri/src/vault/mod.rs:160` — `unlock` nigdy nie pisze; kopia przed migracją musi więc powstać leniwie w `save()`, nie przy odblokowaniu.
- `src-tauri/src/vault/mod.rs:102` — `save()` rotuje `.bak`; kopia przed migracją musi powstać przed krokiem (1) rotacji, z bajtów bieżącego pliku głównego (v1).
- `src-tauri/src/vault/data.rs:33-46` — wzorzec `VersionProbe`: najpierw wersja, potem pola; migrator wpina się między nimi.
- `src-tauri/src/vault/commands.rs:183` — testy kształtu JSON wyników to ustalony sposób pilnowania kontraktu z frontendem; nowe komendy idą tym samym wzorcem.
- `context/foundation/infrastructure.md` (Risk Register) — „Copy the encrypted database file to a timestamped backup before any migration; test each release by updating from the previous version with real data”.

## What We're NOT Doing

- Edycja i usuwanie punktu startowego — teraz tylko potwierdzenie przed zapisem; możliwość późniejszej edycji zapisana w roadmapie jako przyszłe usprawnienie (Parked).
- Historia tankowań / lista wpisów — S-03.
- Edycja i usuwanie tankowań — S-04 (model ma już `id` wpisu, żeby S-04 nie wymagał migracji).
- Samodzielny wpis przebiegu — S-06.
- Koszt w zł/100 km, wykres trendu — Parked (PRD Secondary).
- Wymuszanie przez aplikację, że oba baki były pełne w punkcie startowym — tylko tekst na ekranie.
- Odrzucanie dat z przyszłości (nie wpływają na wynik spalania; daty służą tylko do spójności kolejności).
- Runner testów frontendu, `proptest`, nowe zależności do dat (walidacja daty kalendarzowej własną funkcją).
- Zmiana nagłówka/koperty (`envelope_version` zostaje `1`) i parametrów KDF.

## Implementation Approach

Ten sam porządek co w S-01: czysty Rust testowany `cargo test` (Faza 1), cienkie komendy Tauri (Faza 2), UI (Faza 3), dowód na prawdziwej instalacji (Faza 4).

**Reguła spalania (pełna do pełnej):** wynik liczony za każdym razem od zera z punktu startowego i całej listy tankowań (żadnych zapamiętanych wyników — S-04 dostanie przeliczanie za darmo). Dla każdego paliwa osobno: wpisy tego paliwa posortowane po przebiegu; kotwica = przebieg punktu startowego (punkt startowy = wirtualne tankowanie do pełna obu paliw); litry i kwoty kolejnych wpisów sumowane; tankowanie do pełna zamyka okres: `spalanie = suma_litrów / (przebieg − kotwica) × 100`, `koszt okresu = suma_kwot`, kotwica = jego przebieg, sumy zerowane. Dolewka nie daje wyniku spalania. Cena za litr każdego wpisu = kwota / litry. „Najnowsze spalanie” paliwa = wynik tankowania do pełna tego paliwa o najwyższym przebiegu.

**Spójność wpisów (przed zapisem):** nowy wpis N jest przyjmowany tylko gdy:
- data to poprawna data kalendarzowa `YYYY-MM-DD`; przebieg w `1..=9 999 999` km; litry w `0,01..=1000,00` l; kwota w `0,01..=100 000,00` zł;
- `N.przebieg > start.przebieg` i `N.data ≥ start.data`;
- dla każdego istniejącego wpisu E: `E.przebieg < N.przebieg ⇒ E.data ≤ N.data`; `E.przebieg > N.przebieg ⇒ E.data ≥ N.data`; `E.przebieg = N.przebieg ⇒ E.paliwo ≠ N.paliwo i E.data = N.data`.

Wpisy wsteczne (zapomniany paragon) mieszczące się między istniejącymi są dozwolone. Dzięki tym warunkom każdy okres ma dystans > 0 — dzielenie przez zero jest niemożliwe, a funkcja liczenia i tak zwraca brak wyniku dla dystansu 0 (obrona w głąb).

**Jednostki:** przebieg w całych km (`u32`), litry w centylitrach (`u32`), kwoty w groszach (`u64`) — dane bez błędów zmiennoprzecinkowych; `f64` tylko w wyniku (`l/100 km`, cena za litr w groszach). Frontend zamienia wpisany tekst (przecinek albo kropka, maks. 2 miejsca) na liczby całkowite parsowaniem tekstu, bez mnożenia `float`. Wyświetlanie: `pl-PL`, 2 miejsca po przecinku dla spalania, kwot i ceny za litr.

## Critical Implementation Details

- **Kopia przed migracją (State sequencing).** `decode` pliku v1 zwraca dane v2 i informację o migracji; `UnlockedVault` pamięta, że kopia jest należna. Pierwszy `save()` po migracji, zanim cokolwiek zrotuje: zapisuje bajty bieżącego `kmplus.vault` do `kmplus.vault.pre-v2-<unix s>` (zapis z `sync_all`, nazwa unikalna jak w `damaged_path`), dopiero potem zwykła sekwencja `.bak` → `.tmp` → plik główny; znacznik czyszczony tylko po udanej kopii. Dotyczy też ścieżki `restore_backup` (kopia z `.bak` w formacie v1). Odblokowanie bez zapisu nie zmienia żadnego pliku.
- **Zapis bez połowicznego stanu.** Każda komenda zapisująca buduje nową wersję danych na kopii, wywołuje zapis i dopiero po sukcesie podmienia dane w sesji; przy błędzie zapisu dane w pamięci zostają poprzednie (inaczej UI pokazałby wpis, którego nie ma na dysku).
- **Nazwy argumentów Tauri.** Argumenty komend przychodzą z frontendu w camelCase; złożone wejście przekazujemy jako jeden obiekt `input` ze strukturą `#[serde(rename_all = "camelCase")]`, żeby nazwy pól były w jednym miejscu i w rejestrze kontraktów.

## Phase 1: Model danych v2, migracja i reguła spalania (Rust, bez Tauri)

### Overview

Format ładunku v2 z punktem startowym i tankowaniami, migrator v1 → v2 z leniwą kopią przed migracją, transakcyjna aktualizacja danych sejfu oraz czysty moduł `fuel` z walidacją i regułą pełna-do-pełnej — wszystko pokryte `cargo test`, w tym testem wyroczni na próbce z ręcznie policzonymi wynikami.

### Changes Required:

#### 1. Typy domenowe i reguła

**File**: `src-tauri/src/fuel/mod.rs` (nowy; podział na `model.rs` / `rules.rs` według uznania)

**Intent**: Czysty moduł (bez Tauri, bez IO) z typami zapisywanymi w sejfie, walidacją punktu startowego i wpisu oraz liczeniem wyników według reguł z Implementation Approach.

**Contract**:
- `Fuel` = `Lpg` | `Petrol` (serde: `"lpg"`, `"petrol"`).
- `StartPoint { date: String /* YYYY-MM-DD */, odometer_km: u32 }`.
- `FillUp { id: u64, fuel: Fuel, date: String, odometer_km: u32, litres_cl: u32, amount_grosze: u64, full_tank: bool }`; `NewFillUp` = to samo bez `id`.
- `validate_start_point(&StartPoint) -> Result<(), StartPointRejection>` (`InvalidDate`, `OdometerOutOfRange`; przebieg startowy w `0..=9 999 999`).
- `validate_fill_up(&StartPoint, &[FillUp], &NewFillUp) -> Result<(), FillUpRejection>`; `FillUpRejection` = `InvalidDate` | `OdometerOutOfRange` | `LitresOutOfRange` | `AmountOutOfRange` | `DateBeforeStart` | `OdometerNotAfterStart` | `SameFuelSameOdometer { date }` | `OrderConflict { date, odometer_km }` (wskazuje pierwszy kolidujący wpis; równy przebieg innego paliwa z inną datą też jest `OrderConflict`).
- `compute(&StartPoint, &[FillUp]) -> Vec<(u64, FillUpResult)>` (wynik dla każdego wpisu); `FillUpResult` = `Consumption { litres_per_100km: f64, distance_km: u32, period_litres_cl: u64, period_cost_grosze: u64, price_per_litre_grosze: f64 }` | `TopUp { price_per_litre_grosze: f64 }`; dystans 0 → wynik `TopUp` (brak spalania), bez dzielenia (obrona w głąb, choć walidacja to wyklucza).
- `latest_consumption(&StartPoint, &[FillUp], Fuel) -> Option<(FillUp, FillUpResult)>` — tankowanie do pełna danego paliwa o najwyższym przebiegu i jego wynik.
- Funkcja poprawności daty kalendarzowej (lata przestępne) bez nowego crate'a; porównania dat leksykograficzne na `YYYY-MM-DD`.

#### 2. Format ładunku v2 i migrator

**File**: `src-tauri/src/vault/data.rs`

**Intent**: Podbić format do v2 z polami domenowymi i przenosić pliki v1 w pamięci, zgłaszając, że migracja nastąpiła.

**Contract**:
- `CURRENT_FORMAT_VERSION = 2`; `VaultData { format_version, created_at, start_point: Option<StartPoint>, fill_ups: Vec<FillUp>, next_fill_up_id: u64 }` (`next_fill_up_id` startuje od `1`).
- `decode(plaintext) -> Result<Decoded, DecodeError>`, gdzie `Decoded { data: VaultData, migrated_from: Option<u32> }`; v1 `{format_version:1, created_at}` → v2 z `start_point: None`, `fill_ups: []`, `next_fill_up_id: 1`, `migrated_from: Some(1)`; v2 → `migrated_from: None`; `> 2` → `TooNew`; v2 bez wymaganych pól → `Malformed` (bez `#[serde(default)]` na polach v2 — brakujące pole w v2 to uszkodzenie, nie wartość domyślna).
- Migrator jako osobna funkcja `migrate_v1_to_v2`, żeby kolejne wersje dokładały kroki łańcuchem.

#### 3. Kopia przed migracją i transakcyjna aktualizacja

**File**: `src-tauri/src/vault/mod.rs`

**Intent**: Zapisać nietkniętą kopię pliku v1 przed pierwszym zapisem v2 oraz dać komendom jeden bezpieczny sposób zmiany danych.

**Contract**:
- Stała `PRE_MIGRATION_PREFIX = "kmplus.vault.pre-v"`; plik `kmplus.vault.pre-v<CURRENT_FORMAT_VERSION>-<unix s>` (z sufiksem `-<n>` przy kolizji), nigdy nie usuwany przez aplikację; opis w komentarzu modułu obok pozostałych plików.
- `UnlockedVault` dostaje pole ze znacznikiem należnej kopii (ustawiane z `Decoded::migrated_from` w `try_open`); `save()` realizuje kolejność z Critical Implementation Details.
- `UnlockedVault::commit(&mut self, next: VaultData) -> Result<(), VaultError>` — podmienia dane, zapisuje, a przy błędzie zapisu przywraca poprzednie dane w pamięci.
- `VaultData::new` tworzy pusty v2.

#### 4. Rejestracja modułu

**File**: `src-tauri/src/lib.rs`

**Intent**: Dołączyć `mod fuel;` (komendy dochodzą w Fazie 2).

**Contract**: kompilacja bez ostrzeżeń o martwym kodzie blokujących build (dopuszczalne `pub` API).

#### 5. Testy

**File**: `src-tauri/src/fuel/tests.rs` (nowy) i `src-tauri/src/vault/tests.rs`

**Intent**: Nazwane testy z Testing Strategy oraz test wyroczni na próbce poniżej.

**Contract**: test `oracle_sample_matches_hand_computed_results` odtwarza tabelę „Próbka wyroczni” i porównuje każdy wynik z tolerancją `1e-9` (wartości dokładne, nie zaokrąglone) oraz `latest_consumption` (Gaz → wpis 9, Benzyna → wpis 10). Istniejące testy `newer_format_version_is_too_new` i podobne aktualizowane do wersji 2/3.

**Próbka wyroczni** — start: `2026-01-01`, `100 000 km` (oba baki pełne):

| # | Data | Paliwo | Przebieg | Litry | Kwota zł | Do pełna | Oczekiwany wynik |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 2026-01-05 | Gaz | 100 400 | 40,00 | 120,00 | tak | 400 km, 40,00 l → **10,00** l/100 km; koszt 120,00; 3,00 zł/l |
| 2 | 2026-01-08 | Benzyna | 100 600 | 10,00 | 65,00 | nie | dolewka; 6,50 zł/l |
| 3 | 2026-01-10 | Gaz | 100 750 | 20,00 | 62,00 | nie | dolewka; 3,10 zł/l |
| 4 | 2026-01-12 | Gaz | 100 900 | 25,00 | 75,00 | tak | 500 km, 45,00 l → **9,00**; koszt 137,00; 3,00 zł/l |
| 5 | 2026-01-12 | Benzyna | 100 900 | 30,00 | 195,00 | tak | 900 km, 40,00 l → **4,444…** (4,44); koszt 260,00; 6,50 zł/l |
| 6 | 2026-01-20 | Gaz | 101 350 | 46,35 | 141,37 | tak | 450 km, 46,35 l → **10,30**; koszt 141,37; 3,0500… (3,05) zł/l |
| 7 | 2026-01-25 | Benzyna | 101 500 | 5,00 | 33,00 | nie | dolewka; 6,60 zł/l |
| 8 | 2026-01-28 | Gaz | 101 800 | 12,00 | 36,60 | nie | dolewka; 3,05 zł/l |
| 9 | 2026-02-02 | Gaz | 102 010 | 30,00 | 91,50 | tak | 660 km, 42,00 l → **6,3636…** (6,36); koszt 128,10; 3,05 zł/l |
| 10 | 2026-02-05 | Benzyna | 102 300 | 7,77 | 51,28 | tak | 1 400 km, 12,77 l → **0,9121…** (0,91); koszt 84,28; 6,5997… (6,60) zł/l |

Wpisy 4 i 5: ten sam przebieg i data, różne paliwa — dozwolone. Test wyroczni wstawia wpisy w kolejności przemieszanej (np. 10, 1, 7, 4, …), żeby potwierdzić niezależność od kolejności wstawiania.

### Success Criteria:

#### Automated Verification:

- Testy reguły i sejfu przechodzą: `cargo test` w `src-tauri/`
- Test wyroczni przechodzi: `cargo test oracle_sample_matches_hand_computed_results` w `src-tauri/`
- Projekt Rusta się kompiluje: `cargo check` w `src-tauri/`

#### Manual Verification:

- Użytkownik potwierdza, że wyniki w tabeli „Próbka wyroczni” zgadzają się z tym, jak liczy jego arkusz (albo podmienia próbkę na fragment prawdziwych danych z arkusza z wynikami z arkusza)
- Przegląd listy testów potwierdza pokrycie każdego punktu z Testing Strategy → Unit Tests

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 2: Komendy Tauri

### Overview

Trzy komendy nad sesją sejfu, z wynikami w ustalonym wzorcu `kind`, testami kształtu JSON i aktualizacją rejestru kontraktów.

### Changes Required:

#### 1. Komendy paliwowe

**File**: `src-tauri/src/fuel/commands.rs` (nowy), `src-tauri/src/vault/commands.rs`, `src-tauri/src/lib.rs`

**Intent**: Wystawić frontendowi odczyt stanu paliw, zapis punktu startowego i dodanie tankowania; wszystkie działają tylko na odblokowanej sesji.

**Contract**:
- `lock_session` w `vault/commands.rs` staje się `pub(crate)`; brak odblokowanej sesji → `Err("dane są zablokowane; odblokuj aplikację")`.
- Komendy synchroniczne w treści (bez KDF), ale `async` jak pozostałe; zapis przez `UnlockedVault::commit`. Błąd zapisu → `Err` z komunikatem `VaultError`.
- `fuel_overview() -> FuelOverview` = `{ startPoint: { date, odometerKm } | null, lpg: LatestConsumption | null, petrol: LatestConsumption | null }`; `LatestConsumption` = `{ date, odometerKm, litresPer100Km, distanceKm, periodCostGrosze }`.
- `set_start_point(date: string, odometerKm: number) -> SetStartPointResult` = `{ kind: "ok" } | { kind: "alreadySet" } | { kind: "invalid", reason: "invalidDate" | "odometerOutOfRange" }`.
- `add_fill_up(input: { fuel: "lpg" | "petrol", date, odometerKm, litresCl, amountGrosze, fullTank }) -> AddFillUpResult` = `{ kind: "saved", result: FillUpResult } | { kind: "noStartPoint" } | { kind: "rejected", reason: FillUpRejection }`.
- `FillUpResult` = `{ kind: "consumption", litresPer100Km, distanceKm, periodLitresCl, periodCostGrosze, pricePerLitreGrosze } | { kind: "topUp", pricePerLitreGrosze }`.
- `FillUpRejection` = `{ kind: "invalidDate" | "odometerOutOfRange" | "litresOutOfRange" | "amountOutOfRange" | "dateBeforeStart" | "odometerNotAfterStart" } | { kind: "sameFuelSameOdometer", date } | { kind: "orderConflict", date, odometerKm }`.
- Rejestracja `fuel_overview`, `set_start_point`, `add_fill_up` w `generate_handler!`.
- Testy kształtu JSON dla każdego wariantu, wzorem `commands.rs:183-229`.

#### 2. Rejestr nazw nośnych

**File**: `docs/reference/contract-surfaces.md`

**Intent**: Zapisać nowe nazwy, których zmiana psuje dane lub frontend.

**Contract**: `format_version` = `2`; plik `kmplus.vault.pre-v<N>-<unix s>` (`PRE_MIGRATION_PREFIX`); nowa sekcja „Vault payload” z nazwami pól JSON v2 (`start_point`, `fill_ups`, `next_fill_up_id`, pola `FillUp`, wartości `fuel` `lpg`/`petrol`) i jednostkami (km, centylitry, grosze, `YYYY-MM-DD`); trzy komendy w tabeli „Tauri commands” z argumentami i wartościami `kind`.

### Success Criteria:

#### Automated Verification:

- Projekt Rusta się kompiluje: `cargo check` w `src-tauri/`
- Testy (w tym kształtu JSON nowych wyników) przechodzą: `cargo test` w `src-tauri/`
- Trzy komendy są zarejestrowane: `grep -E "fuel_overview|set_start_point|add_fill_up" src-tauri/src/lib.rs` zwraca wszystkie trzy

#### Manual Verification:

- `docs/reference/contract-surfaces.md` wymienia `format_version` 2, plik `pre-v`, pola ładunku v2 z jednostkami i trzy nowe komendy z wartościami `kind`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 3: Punkt startowy, formularz tankowania i wynik w UI

### Overview

Powłoka po odblokowaniu prowadzi przez punkt startowy (z potwierdzeniem), a potem pokazuje karty spalania obu paliw, formularz tankowania i wynik ostatniego zapisu.

### Changes Required:

#### 1. Typowane wywołania i formatowanie

**File**: `src/fuel/api.ts`, `src/fuel/format.ts` (nowe)

**Intent**: Typy TS odpowiadające kształtom z Fazy 2 i opakowania `invoke`; dokładna zamiana tekstu na liczby całkowite i formatowanie `pl-PL`.

**Contract**:
- `fuelOverview()`, `setStartPoint(date, odometerKm)`, `addFillUp(input)` + typy unii z polem `kind`; `errorMessage` z `src/vault/api.ts` używany ponownie.
- `parseDecimal2(text): number | null` — akceptuje `,` lub `.`, maks. 2 miejsca po separatorze, bez znaku i bez separatorów tysięcy; zwraca wartość × 100 jako liczbę całkowitą wyliczoną z części tekstu (np. `"46,35"` → `4635`, `"7,7"` → `770`, `"5"` → `500`, `"1,234"` → `null`).
- `parseKm(text): number | null` — tylko cyfry (spacje dozwolone i ignorowane).
- Formatery: spalanie `l/100 km` i ceny/kwoty z 2 miejscami (`Intl.NumberFormat("pl-PL")`), data `YYYY-MM-DD` → `DD.MM.YYYY`.

#### 2. Ekran punktu startowego

**File**: `src/fuel/StartPointScreen.tsx` (nowy)

**Intent**: Jednorazowe podanie daty i przebiegu z krokiem potwierdzenia.

**Contract**: pola: data (`<input type="date">`), przebieg (km); tekst: „Podaj datę i przebieg chwili, w której oba baki — gaz i benzyna — były zatankowane do pełna. Punktu startowego nie można później zmienić.”; „Dalej” → podsumowanie „Punkt startowy: DD.MM.RRRR, N km. Czy dane są poprawne?” z przyciskami „Zapisz” i „Popraw”; `invalid` → komunikat przy polu; `alreadySet` → przeładowanie stanu.

#### 3. Formularz tankowania i wynik

**File**: `src/fuel/FillUpForm.tsx`, `src/fuel/FillUpResultPanel.tsx` (nowe)

**Intent**: Dodanie tankowania i natychmiastowe pokazanie wyniku zapisanego wpisu.

**Contract**:
- Pola: paliwo (Gaz / Benzyna), data (domyślnie dziś), przebieg (km), litry, kwota (zł), „Tankowanie do pełna” (domyślnie zaznaczone); przycisk aktywny, gdy wszystkie pola parsują się poprawnie; przycisk wyłączony w trakcie zapisu.
- `saved` z `consumption` → „Spalanie: X,XX l/100 km na N km · Koszt okresu: Y,YY zł · Cena: Z,ZZ zł/l”; `topUp` → „Dolewka — brak wyniku spalania (okres zamknie tankowanie do pełna) · Cena: Z,ZZ zł/l”. Formularz czyszczony (paliwo i data zostają), karty paliw odświeżone.
- `rejected` → komunikat po polsku przy formularzu, dane formularza zostają: np. `orderConflict` → „Wpis koliduje z tankowaniem z DD.MM.RRRR przy N km (przebieg i data muszą rosnąć razem)”, `sameFuelSameOdometer` → „Tankowanie tego paliwa przy tym przebiegu już istnieje (DD.MM.RRRR)”, `odometerNotAfterStart` / `dateBeforeStart` → odwołanie do punktu startowego; komplet komunikatów dla każdej wartości `kind`.
- Błąd `Err` → komunikat z treścią błędu, dane formularza zostają.

#### 4. Powłoka

**File**: `src/AppShell.tsx`, `src/App.css`

**Intent**: Wczytać `fuelOverview` po odblokowaniu i pokazać ekran punktu startowego albo karty + formularz.

**Contract**: stan ładowania / błąd z ponowieniem (wzorem `VaultGate`); karty „Gaz” i „Benzyna”: `X,XX l/100 km` + „ostatnio DD.MM.RRRR, N km” albo „Brak wyniku — potrzebne tankowanie do pełna”; minimalne style w `App.css`.

### Success Criteria:

#### Automated Verification:

- Frontend się buduje (tsc strict + vite): `npm run build`
- Testy Rusta nadal przechodzą: `cargo test` w `src-tauri/`
- Brak podatności w zależnościach JS: `npm audit`

#### Manual Verification:

- `npm run tauri dev` na czystym katalogu danych: ustawienie hasła → ekran punktu startowego; „Dalej” pokazuje podsumowanie, „Popraw” wraca do pól z danymi, „Zapisz” prowadzi do kart „Brak wyniku” dla obu paliw
- Wpisanie wpisów 1–5 z „Próbki wyroczni” (z przecinkiem jako separatorem): po każdym zapisie wynik i cena zgodne z tabelą (2 miejsca); karty po wpisie 5: Gaz 9,00, Benzyna 4,44
- Odrzucenia: przebieg ≤ startowy, data przed startem, wpis Gazu przy 100 400 km ponownie, wpis z przebiegiem 100 500 i datą 2026-01-20 (konflikt kolejności) — każdy daje czytelny komunikat, a karty się nie zmieniają
- Restart i odblokowanie: karty pokazują te same wartości co przed zamknięciem
- Sejf v1 z S-01 (katalog danych sprzed tej zmiany, np. kopia z zainstalowanej 0.1.3): odblokowanie działa i pokazuje ekran punktu startowego; przed zapisem w katalogu nie ma pliku `pre-v2-*`, po zapisie punktu startowego jest

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 4: Wydanie 0.1.4 i dowód migracji

### Overview

Wydanie 0.1.4 istniejącym pipeline'em (pierwszy zielony przebieg bramki `cargo test` na CI) i aktualizacja zainstalowanej 0.1.3 z prawdziwym sejfem v1. Wszystkie bramki ręczne z `context/deployment/deploy-plan.md` obowiązują: start workflow tylko po wyraźnym „start” użytkownika, publikacja draftu wyłącznie ręką użytkownika.

### Changes Required:

#### 1. Wersja 0.1.4

**File**: `src-tauri/tauri.conf.json`, `package.json`, `src-tauri/Cargo.toml` (+ `Cargo.lock`, `package-lock.json`)

**Intent**: Podbić wersję do 0.1.4 we wszystkich trzech miejscach, commit, push na `main`, uruchomić `release.yml` po „start” użytkownika i zweryfikować draft.

**Contract**: trzy pliki wersji zgodne; tag `app-v0.1.4`.

#### 2. Zapis przebiegu

**File**: `context/changes/first-fillup-consumption/change.md` (sekcja `## Notes`), `context/deployment/deploy-plan.md` (`## Przebieg wykonania`)

**Intent**: Ślad dowodowy: numer runu (z widocznym zielonym krokiem „Rust tests”), SHA-256 `kmplus.vault` przed aktualizacją, po aktualizacji i odblokowaniu bez zapisu oraz nazwa powstałego pliku `pre-v2-*` i jego SHA-256 (równy SHA-256 sprzed aktualizacji).

**Contract**: dopisane wiersze; brak zmian w kodzie.

### Success Criteria:

#### Automated Verification:

- Workflow wydania 0.1.4 kończy się sukcesem, krok „Rust tests” zielony: `gh run view <id>` na runie `release.yml`
- Draft `app-v0.1.4` zawiera instalator, `.sig` i `latest.json`: `gh release view app-v0.1.4`

#### Manual Verification:

- Przed aktualizacją: SHA-256 `%APPDATA%\io.github.m-swietek.kmplus\kmplus.vault` zanotowany na zainstalowanej 0.1.3
- Aktualizacja 0.1.3 → 0.1.4 przez baner; to samo hasło odblokowuje; pokazuje się ekran punktu startowego; SHA-256 `kmplus.vault` po odblokowaniu bez zapisu identyczny jak przed aktualizacją
- Po zapisie punktu startowego istnieje `kmplus.vault.pre-v2-<…>` o SHA-256 równym zanotowanemu sprzed aktualizacji; dodanie jednego tankowania do pełna pokazuje spalanie i koszt; po restarcie wynik nadal widoczny

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Testing Strategy

### Unit Tests:

`cargo test`, reguła na czystych danych, sejf na `TempDir` z tanimi parametrami KDF:

- `oracle_sample_matches_hand_computed_results` — tabela „Próbka wyroczni”, kolejność wstawiania przemieszana
- `first_full_fill_up_after_start_gives_result` — dla gazu i dla benzyny (punkt startowy = pełne oba baki)
- `top_up_never_gives_consumption` i `top_up_litres_count_toward_next_full_period`
- `fuels_are_paired_only_with_same_fuel` — przeplatane paliwa nie wpływają na swoje okresy
- `backdated_entry_between_existing_recomputes_periods` — wstawienie pełnego tankowania wstecz dzieli okres
- `zero_distance_never_divides` — funkcja liczenia na danych z pominięciem walidacji
- `latest_consumption_is_highest_odometer_full_fill_up` i `latest_consumption_none_with_only_top_ups`
- Walidacja: `rejects_odometer_not_after_start`, `rejects_date_before_start`, `rejects_same_fuel_same_odometer`, `allows_other_fuel_same_odometer_same_date`, `rejects_other_fuel_same_odometer_other_date`, `rejects_order_conflict_both_directions`, `rejects_out_of_range_values` (granice: 0 i 1 cl, 100 000 i 100 001 cl, 0 i 1 grosz, 9 999 999 i 10 000 000 km), `rejects_invalid_calendar_dates` (`2026-02-29` odrzucone, `2028-02-29` przyjęte, `2026-13-01`, `2026-1-5`)
- Start: `validate_start_point_bounds` (0 km przyjęte)
- Migracja: `v1_payload_migrates_to_empty_v2`, `v2_missing_field_is_malformed`, `newer_format_version_is_too_new` (teraz 3)
- Kopia: `unlock_of_v1_vault_writes_nothing`, `first_save_after_migration_writes_pre_v2_copy_of_v1_bytes`, `second_save_writes_no_further_copy`, `restore_backup_from_v1_then_save_writes_pre_v2_copy`, `v2_vault_save_writes_no_pre_migration_copy`
- Transakcja: `commit_failure_keeps_previous_data_in_memory` (np. katalog danych usunięty lub plik `.tmp` zablokowany katalogiem o tej nazwie)
- Kształt JSON: każdy wariant `FuelOverview`, `SetStartPointResult`, `AddFillUpResult`, `FillUpResult`, `FillUpRejection` (Faza 2)

### Integration Tests:

- Brak automatycznych testów Tauri/UI (frontend bez runnera); ścieżkę komend i UI sprawdzają scenariusze ręczne Fazy 3 i 4.

### Manual Testing Steps:

1. Czysty katalog danych → hasło → punkt startowy z potwierdzeniem → karty „Brak wyniku”.
2. Wpisy 1–5 z próbki wyroczni → wyniki i karty zgodne z tabelą.
3. Cztery przypadki odrzucenia → komunikaty, brak zmian.
4. Restart → te same wartości.
5. Sejf v1 → kopia `pre-v2-*` dopiero po pierwszym zapisie.
6. Wydanie 0.1.4 i aktualizacja z 0.1.3 z porównaniem SHA-256.

## Performance Considerations

- Wyniki liczone od zera przy każdym odczycie i zapisie: sortowanie kilkuset–kilku tysięcy wpisów jednego auta jest pomijalne. Zapis przepisuje cały plik (jak w S-01) — bez znaczenia przy tej skali.

## Migration Notes

- Pierwsza zmiana `format_version` (1 → 2). Plik v1 migrowany w pamięci przy odblokowaniu/przywróceniu; na dysk trafia dopiero przy pierwszym zapisie, poprzedzonym kopią `kmplus.vault.pre-v2-<unix s>` (bajty pliku v1, nigdy nie usuwana).
- Powrót do 0.1.3 po zapisie v2 nie jest możliwy przez updater (tylko do przodu); 0.1.3 na pliku v2 pokaże „dane z nowszej wersji” i niczego nie zmieni. Ręczny ratunek: plik `pre-v2-*` przeniesiony na `kmplus.vault` (z utratą wpisów v2).
- Wzorzec dla kolejnych zmian formatu: nowy krok `migrate_vN_to_vN+1`, nowa nazwa `pre-v<N+1>-*` i test „unlock nie pisze, pierwszy zapis robi kopię”.

## References

- Roadmap: `context/foundation/roadmap.md` (S-02, Open Roadmap Question #1, Parked)
- PRD: `context/foundation/prd.md` (US-01, FR-002, FR-003, FR-007, §Business Logic, §Guardrails)
- Plan S-01 i dług migracyjny: `context/archive/2026-10-09-password-unlocked-vault/plan.md` (Migration Notes)
- Ryzyka migracji: `context/foundation/infrastructure.md` (Risk Register)
- Rytuał wydań i PATH dla cargo: `context/deployment/deploy-plan.md:12`, `## Przebieg wykonania`
- Zapis i rotacja sejfu: `src-tauri/src/vault/mod.rs:102-123`; dekodowanie wersji: `src-tauri/src/vault/data.rs:33-46`
- Wzorzec komend i testów kształtu: `src-tauri/src/vault/commands.rs:17,170,183-229`; rejestracja: `src-tauri/src/lib.rs:12-17`
- Rejestr kontraktów: `docs/reference/contract-surfaces.md`

## Progress

> Convention: `- [ ]` pending, `- [x]` done. Append ` — <commit sha>` when a step lands. Do not rename step titles. See `references/progress-format.md`.

### Phase 1: Model danych v2, migracja i reguła spalania (Rust, bez Tauri)

#### Automated

- [x] 1.1 Testy reguły i sejfu przechodzą: `cargo test` w `src-tauri/` — c2082f8
- [x] 1.2 Test wyroczni przechodzi: `cargo test oracle_sample_matches_hand_computed_results` w `src-tauri/` — c2082f8
- [x] 1.3 Projekt Rusta się kompiluje: `cargo check` w `src-tauri/` — c2082f8

#### Manual

- [x] 1.4 Użytkownik potwierdza, że wyniki w tabeli „Próbka wyroczni” zgadzają się z tym, jak liczy jego arkusz (albo podmienia próbkę na fragment prawdziwych danych z arkusza z wynikami z arkusza) — c2082f8
- [x] 1.5 Przegląd listy testów potwierdza pokrycie każdego punktu z Testing Strategy → Unit Tests — c2082f8

### Phase 2: Komendy Tauri

#### Automated

- [x] 2.1 Projekt Rusta się kompiluje: `cargo check` w `src-tauri/` — 92064b0
- [x] 2.2 Testy (w tym kształtu JSON nowych wyników) przechodzą: `cargo test` w `src-tauri/` — 92064b0
- [x] 2.3 Trzy komendy są zarejestrowane: `grep -E "fuel_overview|set_start_point|add_fill_up" src-tauri/src/lib.rs` zwraca wszystkie trzy — 92064b0

#### Manual

- [x] 2.4 `docs/reference/contract-surfaces.md` wymienia `format_version` 2, plik `pre-v`, pola ładunku v2 z jednostkami i trzy nowe komendy z wartościami `kind` — 92064b0

### Phase 3: Punkt startowy, formularz tankowania i wynik w UI

#### Automated

- [x] 3.1 Frontend się buduje (tsc strict + vite): `npm run build`
- [x] 3.2 Testy Rusta nadal przechodzą: `cargo test` w `src-tauri/`
- [x] 3.3 Brak podatności w zależnościach JS: `npm audit`

#### Manual

- [x] 3.4 `npm run tauri dev` na czystym katalogu danych: ustawienie hasła → ekran punktu startowego; „Dalej” pokazuje podsumowanie, „Popraw” wraca do pól z danymi, „Zapisz” prowadzi do kart „Brak wyniku” dla obu paliw
- [x] 3.5 Wpisanie wpisów 1–5 z „Próbki wyroczni” (z przecinkiem jako separatorem): po każdym zapisie wynik i cena zgodne z tabelą (2 miejsca); karty po wpisie 5: Gaz 9,00, Benzyna 4,44
- [x] 3.6 Odrzucenia: przebieg ≤ startowy, data przed startem, wpis Gazu przy 100 400 km ponownie, wpis z przebiegiem 100 500 i datą 2026-01-20 (konflikt kolejności) — każdy daje czytelny komunikat, a karty się nie zmieniają
- [x] 3.7 Restart i odblokowanie: karty pokazują te same wartości co przed zamknięciem
- [x] 3.8 Sejf v1 z S-01 (katalog danych sprzed tej zmiany, np. kopia z zainstalowanej 0.1.3): odblokowanie działa i pokazuje ekran punktu startowego; przed zapisem w katalogu nie ma pliku `pre-v2-*`, po zapisie punktu startowego jest

### Phase 4: Wydanie 0.1.4 i dowód migracji

#### Automated

- [ ] 4.1 Workflow wydania 0.1.4 kończy się sukcesem, krok „Rust tests” zielony: `gh run view <id>` na runie `release.yml`
- [ ] 4.2 Draft `app-v0.1.4` zawiera instalator, `.sig` i `latest.json`: `gh release view app-v0.1.4`

#### Manual

- [ ] 4.3 Przed aktualizacją: SHA-256 `%APPDATA%\io.github.m-swietek.kmplus\kmplus.vault` zanotowany na zainstalowanej 0.1.3
- [ ] 4.4 Aktualizacja 0.1.3 → 0.1.4 przez baner; to samo hasło odblokowuje; pokazuje się ekran punktu startowego; SHA-256 `kmplus.vault` po odblokowaniu bez zapisu identyczny jak przed aktualizacją
- [ ] 4.5 Po zapisie punktu startowego istnieje `kmplus.vault.pre-v2-<…>` o SHA-256 równym zanotowanemu sprzed aktualizacji; dodanie jednego tankowania do pełna pokazuje spalanie i koszt; po restarcie wynik nadal widoczny
