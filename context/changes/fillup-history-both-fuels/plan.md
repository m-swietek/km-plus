# Historia tankowań i spalanie obu paliw — Implementation Plan

## Overview

S-03 z roadmapy (US-01, FR-007): użytkownik przegląda historię tankowań osobno dla gazu i benzyny, z wynikiem każdego wpisu, a karty paliw pokazują bieżące spalanie — wynik ostatniego zamkniętego okresu oraz średnią od punktu startowego — także przy przeplatanych tankowaniach obu paliw i częściowych dolewkach. Paliwo bez wyniku ma rozróżniony komunikat („brak wpisów” vs „potrzebne tankowanie do pełna”), nigdy szacunek. Zmiana kończy się wydaniem 0.1.5.

## Current State Analysis

Po S-02 (wersja 0.1.4, `research.md` w tym folderze):

- Reguła pełna-do-pełnej per paliwo jest gotowa i przetestowana: `compute` (`src-tauri/src/fuel/mod.rs:183-224`) zwraca wynik dla każdego wpisu; przeplatanie i dolewki pokrywają testy `tests.rs:113,190,205`.
- „Najnowsze spalanie” = wynik tankowania do pełna danego paliwa o najwyższym przebiegu (`latest_consumption`, `mod.rs:228-243`), wystawione przez `fuel_overview` jako `lpg`/`petrol: LatestConsumption | null` (`commands.rs:24-48,214-249`).
- Brak historii: żadna z 7 komend (`lib.rs:15-23`) nie zwraca listy wpisów; frontend nie ma widoku listy.
- `null` w karcie nie rozróżnia „brak wpisów” od „same dolewki” (`AppShell.tsx:88`, `tests.rs:284-293`).
- Format sejfu v2 (`vault/data.rs:12`) zawiera już wszystko, czego potrzeba (`fill_ups` z `id`); wyniki nie są zapisywane.

## Desired End State

- Po odblokowaniu i z ustawionym punktem startowym każda karta paliwa pokazuje jeden z trzech stanów: „Brak wpisów”; „Brak wyniku — potrzebne tankowanie do pełna”; albo spalanie ostatniego zamkniętego okresu (duża liczba, data i przebieg) plus wiersz „średnio od startu: X l/100 km na Y km”.
- Pod formularzem tankowania widać dwie listy historii — Gaz i Benzyna — każda od najnowszego wpisu; wiersz pokazuje datę, przebieg, litry, kwotę, cenę za litr i znacznik pełny/dolewka; wiersz tankowania do pełna, które zamknęło okres, dodatkowo spalanie, dystans i koszt okresu; wiersz dolewki „dolewka” bez liczby spalania. Pusta lista: „Brak tankowań”.
- Po zapisie tankowania karty i historia aktualizują się bez dodatkowej akcji, w jednym odświeżeniu.
- Liczby w kartach, historii i panelu po zapisie pochodzą z tego samego `compute`.
- Wersja 0.1.5 wydana; aktualizacja 0.1.4 → 0.1.5 z prawdziwymi danymi nie zmienia pliku sejfu przy odblokowaniu i pokazuje istniejące wpisy w historii.

### Key Discoveries:

- `src-tauri/src/fuel/mod.rs:200` — wynik `Consumption` tylko dla `full_tank && distance_km > 0`; średnia i „ostatni okres” mają się opierać wyłącznie na wynikach `Consumption`, nie na flagach wpisów.
- `src-tauri/src/fuel/mod.rs:128-174` — walidacja wymusza wspólny wzrost przebiegu i daty, więc sortowanie per paliwo po przebiegu = po dacie; w obrębie jednego paliwa ten sam przebieg jest odrzucany (`mod.rs:169-172`), więc remisów w liście jednego paliwa nie ma przy danych po walidacji (tie-break po `id` zostaje jako zabezpieczenie).
- Suma dystansów zamkniętych okresów paliwa = przebieg ostatniego zamkniętego okresu − przebieg punktu startowego (okresy są ciągłe od startu, `mod.rs:191,209`).
- `src-tauri/src/fuel/commands.rs:139-162` — `From<FillUpResult> for FillUpResultView` do ponownego użycia w wierszach historii.
- `src/AppShell.tsx:69` — `onSaved={fetchOverview}`; rozszerzenie `fuel_overview` daje odświeżenie historii bez nowego hooka.

## What We're NOT Doing

- Edycja i usuwanie wpisów (S-04).
- Jedna wspólna, przeplatana lista obu paliw; sortowanie rosnące; filtrowanie, stronicowanie, wyszukiwanie.
- Pokazywanie sum litrów/kwot otwartego okresu (po ostatnim pełnym) — odrzucone, zbyt łatwo czytane jako wynik.
- Ukrywanie średniej przy jednym zamkniętym okresie — decyzja: średnia widoczna od pierwszego zamkniętego okresu, nawet gdy równa ostatniemu.
- Wykres trendu, koszt zł/100 km (Parked w roadmapie).
- Zmiana formatu sejfu, migracja, nowa komenda Tauri.
- Zmiana reguły parowania lub uproszczenia LPG (rozruch na benzynie liczony do gazu — świadomie).

## Implementation Approach

Pure Rust najpierw: jedna funkcja domenowa liczy dla danego paliwa stan podsumowania (trzy warianty), średnią z zamkniętych okresów i posortowaną historię — z jednego wywołania `compute` na całą listę. Następnie `fuel_overview` zostaje rozszerzone (zamiast nowej komendy), żeby frontend dostawał jeden spójny obraz danych w jednym `invoke` i odświeżał go istniejącym `fetchOverview`. Zmiana kształtu `FuelOverview` jest łamiąca, ale frontend i backend wychodzą razem; zapisujemy ją w `contract-surfaces.md`. UI tylko wyświetla — żadnej logiki parowania ani liczenia średniej po stronie TS.

## Phase 1: Domena — podsumowanie i historia per paliwo

### Overview

Czysta funkcja w `fuel/mod.rs` budująca widok jednego paliwa: stan, ostatni zamknięty okres, średnia od startu, historia od najnowszego. Pokryta `cargo test`, w tym na próbce wyroczni.

### Changes Required:

#### 1. Raport paliwa

**File**: `src-tauri/src/fuel/mod.rs`

**Intent**: Dodać domenowy raport per paliwo zastępujący `latest_consumption`, tak żeby karta i historia miały jedno źródło liczb.

**Contract**:
- `FuelSummary` (domena, bez serde): `NoEntries` (paliwo nie ma żadnego wpisu) | `NoClosedPeriod` (są wpisy, żaden wynik nie jest `Consumption`) | `Consumption { latest: (FillUp, FillUpResult::Consumption), average: AverageConsumption }`.
- `latest` = wpis tego paliwa o najwyższym `(odometer_km, id)` wśród wpisów, których wynik jest `Consumption` (przy danych po walidacji tożsame z dotychczasową definicją „tankowanie do pełna o najwyższym przebiegu”).
- `AverageConsumption { litres_per_100km: f64, distance_km: u32, litres_cl: u64, periods: u32 }`: sumy `period_litres_cl` i `distance_km` po wszystkich wynikach `Consumption` tego paliwa; `litres_per_100km = litres_cl / distance_km` (centylitry/km = l/100 km, jak w `compute`). Litry dolewek po ostatnim pełnym nie wchodzą do średniej (nie należą do żadnego zamkniętego okresu).
- Historia: `Vec<(FillUp, FillUpResult)>` wpisów tego paliwa posortowana malejąco po `(odometer_km, id)`.
- Jedno wywołanie `compute` na raport obu paliw (np. `fuel_reports(start, fill_ups) -> [(Fuel, FuelReport); 2]` albo równoważne) — nazewnictwo dowolne, warunek: oba paliwa i obie części raportu z jednego `compute`.
- `latest_consumption` usunięte, jego testy przeniesione na nową funkcję (bez martwego kodu).

#### 2. Testy reguły

**File**: `src-tauri/src/fuel/tests.rs`

**Intent**: Udowodnić stany, średnią i kolejność, w szczególności dla przeplatanych paliw i dolewek.

**Contract**: nowe/przeniesione testy:
- próbka wyroczni (`oracle_sample`, kolejność wstawiania jak w istniejącym teście): LPG `Consumption`, `latest` = wpis 9, średnia 17 335 cl / 2 010 km = 8,6244… l/100 km, `periods = 4`; benzyna `Consumption`, `latest` = wpis 10, średnia 5 277 cl / 2 300 km = 2,2943… l/100 km, `periods = 2`; historia LPG = ids `[9, 8, 6, 4, 3, 1]`, benzyny = `[10, 7, 5, 2]`, każdy wiersz z wynikiem równym `compute` dla tego id;
- `NoEntries` dla pustej listy i dla paliwa bez wpisów, gdy drugie paliwo ma wpisy;
- `NoClosedPeriod` przy samych dolewkach danego paliwa (przeniesiony `latest_consumption_none_with_only_top_ups`);
- jeden zamknięty okres: średnia równa wynikowi ostatniego okresu, `periods = 1`;
- dolewka po ostatnim pełnym nie zmienia `latest` ani średniej (przeniesiony `latest_consumption_is_highest_odometer_full_fill_up` + asercja średniej);
- wpis wsteczny pomiędzy istniejącymi zmienia średnią i kolejność historii zgodnie z przeliczeniem.

### Success Criteria:

#### Automated Verification:

- Testy domeny przechodzą: `cargo test fuel::tests` w `src-tauri/`
- Cały zestaw testów przechodzi: `cargo test` w `src-tauri/`
- Brak ostrzeżeń kompilacji o martwym kodzie: `cargo check` w `src-tauri/`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 2: Kontrakt — rozszerzone `fuel_overview`

### Overview

DTO i kształt JSON nowego `FuelOverview`, testy kształtu, rejestr kontraktu i typy TS.

### Changes Required:

#### 1. DTO i komenda

**File**: `src-tauri/src/fuel/commands.rs`

**Intent**: `fuel_overview` zwraca dla każdego paliwa stan, ostatni okres, średnią i historię z raportu z Fazy 1.

**Contract** (camelCase, unie z `kind`, jak istniejące DTO):
- `FuelOverview = { startPoint: StartPointView | null, lpg: FuelView | null, petrol: FuelView | null }`; `lpg`/`petrol` są `null` wtedy i tylko wtedy, gdy `startPoint` jest `null`.
- `FuelView = { summary: FuelSummaryView, history: HistoryRow[] }`.
- `FuelSummaryView = { kind: "noEntries" } | { kind: "noClosedPeriod" } | { kind: "consumption", latest: LatestConsumption, average: AverageConsumption }`.
- `LatestConsumption` bez zmian: `{ date, odometerKm, litresPer100Km, distanceKm, periodCostGrosze }`.
- `AverageConsumption = { litresPer100Km, distanceKm, litresCl, periods }`.
- `HistoryRow = { id, date, odometerKm, litresCl, amountGrosze, fullTank, result: FillUpResultView }` (bez pola `fuel` — lista jest per paliwo).
- `overview()` przestaje używać `latest_consumption`; testy kształtu JSON dla wszystkich nowych typów (w tym `noEntries`, `noClosedPeriod`, `consumption`, wiersz dolewki i wiersz pełnego) oraz istniejący test `saved_fill_ups_get_increasing_ids_and_feed_the_overview` dostosowany do nowego kształtu.

#### 2. Rejestr kontraktu

**File**: `docs/reference/contract-surfaces.md`

**Intent**: Zaktualizować wiersz `fuel_overview` (linia 60) do nowego kształtu i zaznaczyć, że zmiana kształtu odpowiedzi nie dotyka formatu sejfu.

**Contract**: wiersz tabeli komend.

#### 3. Typy frontendu

**File**: `src/fuel/api.ts`

**Intent**: Odbić nowe typy (`FuelView`, `FuelSummaryView`, `AverageConsumption`, `HistoryRow`) i zaktualizować `FuelOverview`; wrapper `fuelOverview()` bez zmian.

**Contract**: typy TS zgodne 1:1 z DTO; `FillUpResult` używany ponownie w `HistoryRow.result`.

### Success Criteria:

#### Automated Verification:

- Testy kształtu JSON i komend przechodzą: `cargo test fuel::commands` w `src-tauri/`
- Cały zestaw testów przechodzi: `cargo test` w `src-tauri/`
- Frontend kompiluje się z nowymi typami (AppShell dostosowany minimalnie, żeby build przechodził): `npm run build`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 3: UI — karty i listy historii

### Overview

Karty paliw z trzema stanami i średnią, dwie listy historii pod formularzem, odświeżanie po zapisie istniejącym `fetchOverview`.

### Changes Required:

#### 1. Karty paliw

**File**: `src/AppShell.tsx`

**Intent**: `FuelCard` renderuje `FuelSummaryView`: `noEntries` → „Brak wpisów”; `noClosedPeriod` → „Brak wyniku — potrzebne tankowanie do pełna”; `consumption` → wartość `latest.litresPer100Km`, meta „ostatnio <data>, <przebieg>” i wiersz „średnio od startu: <l/100 km> na <km>”.

**Contract**: wyłącznie istniejące formatery z `src/fuel/format.ts` (`formatConsumption`, `formatKm`, `formatDate`); żadnego liczenia w TS.

#### 2. Listy historii

**File**: `src/fuel/FillUpHistory.tsx` (nowy), `src/AppShell.tsx`, `src/App.css`

**Intent**: Komponent listy jednego paliwa (nagłówek z `FUEL_LABELS`), wiersze w kolejności z backendu: data, przebieg, litry, kwota, cena/l, „do pełna”/„dolewka”; dla `result.kind === "consumption"` dodatkowo spalanie, dystans i koszt okresu; pusta lista „Brak tankowań”. Dwie listy obok siebie pod formularzem (siatka jak `.fuel-cards`, jedna kolumna przy wąskim oknie), style również w trybie ciemnym.

**Contract**: props `{ fuel: Fuel; rows: HistoryRow[] }`; formatery z `format.ts` (w tym `formatMoney`, `formatPricePerLitre`); litry z centylitrów formatowane z 2 miejscami (nowy formater w `format.ts`, jeśli brak).

### Success Criteria:

#### Automated Verification:

- Frontend kompiluje się bez nieużywanych importów: `npm run build`
- Testy backendu nadal przechodzą: `cargo test` w `src-tauri/`

#### Manual Verification:

- `npm run tauri dev` na czystym katalogu danych (prawdziwy `kmplus.vault` przeniesiony na bok przed testem i przywrócony po nim): po punkcie startowym (2026-01-01, 100 000 km) obie karty pokazują „Brak wpisów”, obie listy „Brak tankowań”
- Po dolewce LPG (2026-01-03, 100 200 km, 20 l, 60 zł): karta Gaz „Brak wyniku — potrzebne tankowanie do pełna”, karta Benzyna nadal „Brak wpisów”; lista Gaz ma wiersz „dolewka” bez spalania
- Po pełnym LPG (2026-01-05, 100 400 km, 20 l, 60 zł): karta Gaz 10,00 l/100 km i „średnio od startu: 10,00 l/100 km na 400 km”; lista Gaz ma pełny wiersz na górze ze spalaniem 10,00 l/100 km, dystansem 400 km i kosztem okresu 120,00 zł
- Po pełnej benzynie (2026-01-06, 100 500 km, 5 l, 32,50 zł) i pełnym LPG (2026-01-10, 100 900 km, 45 l, 135 zł): karta Benzyna 1,00 l/100 km na 500 km; karta Gaz 9,00 l/100 km i średnia 9,44 l/100 km na 900 km; wpisy benzyny nie pojawiają się w liście Gaz
- Karty i listy aktualizują się po każdym zapisie bez przeładowania; po restarcie i odblokowaniu widok identyczny; tryb ciemny czytelny

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 4: Wydanie 0.1.5

### Overview

Wydanie istniejącym pipeline'em według rytuału z S-02 i `context/deployment/deploy-plan.md`: start workflow tylko po wyraźnym „start” użytkownika, publikacja draftu wyłącznie ręką użytkownika. Format sejfu się nie zmienia, więc dowodem jest aktualizacja 0.1.4 → 0.1.5 bez zmiany pliku przy odblokowaniu.

### Changes Required:

#### 1. Wersja 0.1.5

**File**: `src-tauri/tauri.conf.json`, `package.json`, `src-tauri/Cargo.toml` (+ `Cargo.lock`, `package-lock.json`)

**Intent**: Podbić wersję do 0.1.5 we wszystkich miejscach, commit, push na `main`, uruchomić `release.yml` po „start” użytkownika i zweryfikować draft.

**Contract**: wersje zgodne; tag `app-v0.1.5`.

#### 2. Zapis przebiegu

**File**: `context/changes/fillup-history-both-fuels/change.md` (`## Notes`), `context/deployment/deploy-plan.md` (`## Przebieg wykonania`)

**Intent**: Ślad dowodowy: numer runu z zielonym krokiem „Rust tests”, SHA-256 `kmplus.vault` przed aktualizacją i po aktualizacji z odblokowaniem bez zapisu.

**Contract**: dopisane wiersze; brak zmian w kodzie.

### Success Criteria:

#### Automated Verification:

- Workflow wydania 0.1.5 kończy się sukcesem, krok „Rust tests” zielony: `gh run view <id>` na runie `release.yml`
- Draft `app-v0.1.5` zawiera instalator, `.sig` i `latest.json`: `gh release view app-v0.1.5`

#### Manual Verification:

- Przed aktualizacją: SHA-256 `%APPDATA%\io.github.m-swietek.kmplus\kmplus.vault` zanotowany na zainstalowanej 0.1.4
- Aktualizacja 0.1.4 → 0.1.5 przez baner; to samo hasło odblokowuje; SHA-256 `kmplus.vault` po odblokowaniu bez zapisu identyczny jak przed aktualizacją; nie powstaje plik `pre-v*`
- Istniejące tankowania widoczne w listach historii z wynikami; karty pokazują ostatni okres i średnią; przebieg zapisany w `change.md` i `deploy-plan.md`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Testing Strategy

### Unit Tests:

- `cargo test`: raport paliwa na czystych danych (Faza 1) — trzy stany, średnia z zamkniętych okresów, kolejność historii, przeplatanie paliw, dolewki przed i po pełnym, wpis wsteczny; próbka wyroczni z ręcznie policzonymi średnimi (LPG 17 335 cl / 2 010 km, benzyna 5 277 cl / 2 300 km).
- Testy kształtu JSON (Faza 2) dla każdego wariantu `FuelSummaryView` i obu rodzajów `HistoryRow`.

### Integration Tests:

- `overview()` na sejfie w `TempDir` z tanimi parametrami KDF (istniejący wzorzec `commands.rs` testy) — po zapisach przez `apply_fill_up` raport zgadza się z oczekiwanym.

### Manual Testing Steps:

1. Scenariusz z Fazy 3 krok po kroku na czystym katalogu danych (prawdziwy sejf odłożony na bok).
2. Restart i odblokowanie — widok bez zmian.
3. Aktualizacja 0.1.4 → 0.1.5 na prawdziwych danych (Faza 4).

## Performance Considerations

Jedno `compute` na wywołanie `fuel_overview` (dotąd dwa, przez `latest_consumption` per paliwo). Liniowe w liczbie wpisów; przy dzienniku jednego auta bez znaczenia.

## Migration Notes

Brak zmiany formatu sejfu (pozostaje v2, `vault/data.rs:12`) — brak migratora i kopii `pre-v*`. Zmienia się wyłącznie kształt odpowiedzi `fuel_overview`, wydawany razem z frontendem.

## References

- Related research: `context/changes/fillup-history-both-fuels/research.md`
- Roadmap: `context/foundation/roadmap.md` (S-03)
- Poprzednia zmiana: `context/archive/2026-10-11-first-fillup-consumption/plan.md`
- Reguła: `src-tauri/src/fuel/mod.rs:183-243`
- Komenda i DTO: `src-tauri/src/fuel/commands.rs:24-48,214-249`
- Karty: `src/AppShell.tsx:76-91`

## Progress

> Convention: `- [ ]` pending, `- [x]` done. Append ` — <commit sha>` when a step lands. Do not rename step titles. See `references/progress-format.md`.

### Phase 1: Domena — podsumowanie i historia per paliwo

#### Automated

- [x] 1.1 Testy domeny przechodzą: `cargo test fuel::tests` w `src-tauri/` — 151309e
- [x] 1.2 Cały zestaw testów przechodzi: `cargo test` w `src-tauri/` — 151309e
- [x] 1.3 Brak ostrzeżeń kompilacji o martwym kodzie: `cargo check` w `src-tauri/` — 151309e

### Phase 2: Kontrakt — rozszerzone `fuel_overview`

#### Automated

- [x] 2.1 Testy kształtu JSON i komend przechodzą: `cargo test fuel::commands` w `src-tauri/` — c419442
- [x] 2.2 Cały zestaw testów przechodzi: `cargo test` w `src-tauri/` — c419442
- [x] 2.3 Frontend kompiluje się z nowymi typami (AppShell dostosowany minimalnie, żeby build przechodził): `npm run build` — c419442

### Phase 3: UI — karty i listy historii

#### Automated

- [x] 3.1 Frontend kompiluje się bez nieużywanych importów: `npm run build` — 13d0b4f
- [x] 3.2 Testy backendu nadal przechodzą: `cargo test` w `src-tauri/` — 13d0b4f

#### Manual

- [x] 3.3 `npm run tauri dev` na czystym katalogu danych (prawdziwy `kmplus.vault` przeniesiony na bok przed testem i przywrócony po nim): po punkcie startowym (2026-01-01, 100 000 km) obie karty pokazują „Brak wpisów”, obie listy „Brak tankowań” — 13d0b4f
- [x] 3.4 Po dolewce LPG (2026-01-03, 100 200 km, 20 l, 60 zł): karta Gaz „Brak wyniku — potrzebne tankowanie do pełna”, karta Benzyna nadal „Brak wpisów”; lista Gaz ma wiersz „dolewka” bez spalania — 13d0b4f
- [x] 3.5 Po pełnym LPG (2026-01-05, 100 400 km, 20 l, 60 zł): karta Gaz 10,00 l/100 km i „średnio od startu: 10,00 l/100 km na 400 km”; lista Gaz ma pełny wiersz na górze ze spalaniem 10,00 l/100 km, dystansem 400 km i kosztem okresu 120,00 zł — 13d0b4f
- [x] 3.6 Po pełnej benzynie (2026-01-06, 100 500 km, 5 l, 32,50 zł) i pełnym LPG (2026-01-10, 100 900 km, 45 l, 135 zł): karta Benzyna 1,00 l/100 km na 500 km; karta Gaz 9,00 l/100 km i średnia 9,44 l/100 km na 900 km; wpisy benzyny nie pojawiają się w liście Gaz — 13d0b4f
- [x] 3.7 Karty i listy aktualizują się po każdym zapisie bez przeładowania; po restarcie i odblokowaniu widok identyczny; tryb ciemny czytelny — 13d0b4f

### Phase 4: Wydanie 0.1.5

#### Automated

- [x] 4.1 Workflow wydania 0.1.5 kończy się sukcesem, krok „Rust tests” zielony: `gh run view <id>` na runie `release.yml`
- [x] 4.2 Draft `app-v0.1.5` zawiera instalator, `.sig` i `latest.json`: `gh release view app-v0.1.5`

#### Manual

- [x] 4.3 Przed aktualizacją: SHA-256 `%APPDATA%\io.github.m-swietek.kmplus\kmplus.vault` zanotowany na zainstalowanej 0.1.4
- [x] 4.4 Aktualizacja 0.1.4 → 0.1.5 przez baner; to samo hasło odblokowuje; SHA-256 `kmplus.vault` po odblokowaniu bez zapisu identyczny jak przed aktualizacją; nie powstaje plik `pre-v*`
- [x] 4.5 Istniejące tankowania widoczne w listach historii z wynikami; karty pokazują ostatni okres i średnią; przebieg zapisany w `change.md` i `deploy-plan.md`
