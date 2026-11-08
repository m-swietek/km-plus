# Poprawka i usunięcie tankowania — Implementation Plan

## Overview

S-04 z roadmapy (US-01, FR-004, FR-005): użytkownik poprawia dowolne pole istniejącego wpisu tankowania (także rodzaj paliwa) albo usuwa wpis po dwuetapowym potwierdzeniu w wierszu historii, a karty i historie obu paliw od razu pokazują spalanie przeliczone z poprawionej listy — identyczne z tym, co dałoby wpisanie tej listy od nowa. Bez zmiany formatu sejfu, bez cofania i bez kosza. Zmiana kończy się wydaniem 0.1.7 (albo kolejnym numerem, jeśli S-05 wyjdzie wcześniej).

## Current State Analysis

Po S-03 (wydane 0.1.5, poprawki recenzji 0.1.6 — `app-v0.1.6` opublikowane, Latest):

- Wyniki są czystą funkcją `(start_point, fill_ups)`: `compute` (`src-tauri/src/fuel/mod.rs:183-225`) i `fuel_reports` (`mod.rs:265-268`) liczą wszystko od zera przy każdym odczycie; nic liczonego nie jest zapisywane (`docs/reference/contract-surfaces.md:40`). Przeliczenie po edycji/usunięciu nie wymaga nowego kodu reguły.
- Zapis idzie przez `UnlockedVault::commit` (`src-tauri/src/vault/mod.rs:163-170`): przy błędzie zapisu w pamięci zostają poprzednie dane; `save` rotuje poprzedni plik do `kmplus.vault.bak` (`vault/mod.rs:148-153`) — to kopia na wypadek uszkodzenia, nie „cofnij”: po usunięciu trzyma stan sprzed usunięcia tylko do następnego zapisu.
- `FillUp.id` jest w formacie v2 od S-02, `next_fill_up_id` gwarantuje brak ponownego użycia id (`vault/data.rs:25-27`). S-04 nie potrzebuje żadnego nowego pola.
- Walidacja nowego wpisu `validate_fill_up(start, existing, new)` (`mod.rs:128-178`) sprawdza parami każdy istniejący wpis. Brak wariantu dla edycji; brak usuwania.
- Komendy (`lib.rs:15-23`): 7, w tym `add_fill_up`; brak komend edycji i usuwania. Wzorzec: `with_vault` + `apply_*` + DTO z `kind` (`fuel/commands.rs:310-391`).
- Frontend: `FillUpForm` (`src/fuel/FillUpForm.tsx`) ma parsowanie tekstu, strażniki u32 i polskie komunikaty odrzuceń, a wynik ostatniego dodania trzyma w lokalnym stanie `saved` (`FillUpForm.tsx:56,179`). `FillUpHistory` (`src/fuel/FillUpHistory.tsx`) renderuje wiersze z `id`, bez akcji. `AppShell` odświeża wszystko przez `fetchOverview` (`src/AppShell.tsx:25-31,77-82`).

## Desired End State

- Każdy wiersz obu list historii ma przyciski „Edytuj” i „Usuń”.
- „Edytuj” ładuje wpis do wspólnego formularza nad historiami: nagłówek „Popraw tankowanie”, pola wypełnione wartościami wpisu (paliwo z listy, w której jest wiersz), przyciski „Zapisz zmiany” i „Anuluj”. Edytowalne są wszystkie pola: paliwo, data, przebieg, litry, kwota, „do pełna”. Zapis obowiązują te same reguły i komunikaty co przy dodawaniu, z pominięciem samego edytowanego wpisu. Wpis zachowuje `id`; po zmianie paliwa przechodzi do drugiej listy i przeliczają się oba paliwa.
- „Usuń” zamienia wiersz na „Usunąć tankowanie z DD.MM.YYYY, X km? [Usuń] [Anuluj]” — dopiero drugie kliknięcie usuwa wpis trwale. Potwierdzenie jest aktywne najwyżej dla jednego wiersza naraz (na całym ekranie).
- Jeden panel statusu pod formularzem opisuje ostatnią akcję: „Zapisano: <paliwo>, <data>” z wynikiem (jak dziś), „Poprawiono: <paliwo>, <data>” z przeliczonym wynikiem poprawionego wpisu, „Usunięto: <paliwo>, <data>, <przebieg>”. Każda akcja zastępuje poprzedni komunikat; wejście w tryb edycji czyści panel.
- Po każdej akcji karty i historie obu paliw odświeżają się bez dodatkowej akcji (istniejące `fetchOverview`), a liczby są równe liczbom z listy wpisanej od nowa.
- Wersja 0.1.7 (albo kolejna po S-05) wydana; aktualizacja z prawdziwymi danymi nie zmienia pliku sejfu przy odblokowaniu.

### Key Discoveries:

- `src-tauri/src/fuel/mod.rs:169-172` — `SameFuelSameOdometer`: walidacja edycji na pełnej liście odrzuciłaby wpis zapisany bez zmian (koliduje sam ze sobą). Edycja musi walidować wobec wszystkich wpisów **poza** edytowanym.
- `src-tauri/src/fuel/mod.rs:152-176` — reguły walidacji są parami (każdy istniejący wpis vs nowy), więc usunięcie wpisu nigdy nie łamie spójności pozostałych; usunięcie nie wymaga walidacji.
- `src-tauri/src/fuel/mod.rs:189` — `compute` sortuje wpisy paliwa po `(odometer_km, id)`, więc pozycja wpisu w `fill_ups` nie wpływa na wyniki; poprawiony wpis może zostać na swoim miejscu w wektorze.
- `src-tauri/src/fuel/tests.rs:134-148` — próbka wyroczni (10 wpisów, ids = numery wierszy) z ręcznie policzonymi wynikami — baza testów równoważności.
- `src-tauri/src/fuel/commands.rs:366-391` — `apply_fill_up`: wzorzec do skopiowania (klon danych → zmiana → `compute` dla wyniku → `commit`).
- `src/AppShell.tsx:76-82` — formularz i historie są rodzeństwem; stan edycji musi żyć nad nimi. S-05 owija ten blok w zakładkę „Paliwo” bez zmian, więc zmiana `AppShell` w S-04 ma być minimalna (wywołanie hooka + propsy).

## What We're NOT Doing

- Cofanie usunięcia, kosz / miękkie usuwanie, zapamiętywanie usuniętych wpisów — decyzja: potwierdzenie w wierszu, usunięcie jest ostateczne.
- Zmiana formatu sejfu, migracja, kopia `pre-v*` — S-04 nie dodaje żadnego pola.
- Edycja w wierszu historii i okno modalne — decyzja: wspólny formularz w trybie edycji.
- Natywne okno potwierdzenia (wtyczka dialog Tauri, `window.confirm`) — potwierdzenie wyłącznie w interfejsie aplikacji.
- Pokazywanie wartości kart „przed → po” — decyzja: jeden panel statusu z wynikiem poprawionego wpisu.
- Zachowywanie niezapisanego szkicu nowego wpisu po kliknięciu „Edytuj” (szkic jest zastępowany).
- Usuwanie wielu wpisów naraz, zaznaczanie, filtrowanie.
- Edycja/usuwanie punktu startowego (Parked w roadmapie), samodzielnego wpisu przebiegu (S-06) i serwisów (S-05).
- Zmiana reguły parowania i uproszczenia LPG (rozruch na benzynie liczony do gazu — świadomie).

## Implementation Approach

Kolejność jak w S-02/S-03: czysta domena w Ruście z testami → komendy i kontrakt → UI → wydanie. Domena dostaje dwie czyste funkcje (podmiana wpisu z walidacją bez samego edytowanego wpisu, usunięcie po `id`), więc całe przeliczanie zostaje w istniejącym `compute`. Komendy powielają wzorzec `apply_fill_up` i zapisują przez `commit`. UI nie liczy niczego: formularz dostaje tryb edycji, wiersze historii akcje z potwierdzeniem, a wspólny stan (wpis w edycji, potwierdzany wiersz, komunikat ostatniej akcji) trzyma nowy hook w `src/fuel/`, żeby `AppShell` zmienił się tylko o wywołanie hooka i propsy (konflikt z zakładkami S-05).

Decyzje UI podjęte przez agenta (niezakwestionowane przez użytkownika): „Edytuj” zastępuje niezapisany szkic nowego wpisu; po „Zapisz zmiany” lub „Anuluj” formularz wraca do trybu dodawania (paliwo zostaje, data wraca na dziś, pozostałe pola puste); usunięcie wpisu, który jest właśnie edytowany, kończy tryb edycji; żaden nieaktualny panel wyniku nie zostaje na ekranie po edycji lub usunięciu.

## Critical Implementation Details

- **State sequencing (walidacja edycji):** zbiór `existing` przekazywany do `validate_fill_up` przy edycji musi wykluczać edytowane `id`; inaczej zapis bez zmian zwraca `sameFuelSameOdometer`. Test `edit_unchanged_entry_is_accepted` pilnuje tego dla każdego wpisu próbki.
- **Weryfikacja ręczna na kopii danych:** `npm run tauri dev` używa tego samego katalogu danych co zainstalowana aplikacja (`%APPDATA%\io.github.m-swietek.kmplus`). Edycja i usuwanie to zapisy — test ręczny Fazy 3 wolno robić tylko po odłożeniu prawdziwych plików na bok (z zanotowanym SHA-256) i trzeba je przywrócić z weryfikacją hasha przed Fazą 4 (powtórka uchybienia F2 z recenzji S-03).
- **Koordynacja z równoległym S-05 (`plan-service`):** ręczna weryfikacja Fazy 3 i cała Faza 4 nie mogą nakładać się w czasie z ręczną weryfikacją Fazy 3 ani z Fazą 4 S-05. Obie zmiany używają tego samego katalogu danych (`%APPDATA%\io.github.m-swietek.kmplus`) i portu 1420 (`strictPort`), a build deweloperski S-05 zapisuje sejf w formacie v3. Przed odłożeniem prawdziwych plików upewnij się, że S-05 nie ma właśnie odłożonego prawdziwego sejfu ani działającego `tauri dev`. Wydania idą po kolei: zalecana kolejność to S-04 scalone i wydane pierwsze, potem S-05 robi rebase na `main`; jeśli kolejność się odwróci, S-04 robi rebase na `main` po wydaniu S-05, powtarza testy automatyczne i bierze kolejny numer wersji.

## Phase 1: Domena — poprawka i usunięcie wpisu

### Overview

Dwie czyste funkcje w `fuel/mod.rs` i testy dowodzące, że po każdej poprawce i każdym usunięciu raporty obu paliw są równe raportom listy wpisanej od nowa.

### Changes Required:

#### 1. Operacje na liście wpisów

**File**: `src-tauri/src/fuel/mod.rs`

**Intent**: Dodać podmianę wpisu (z walidacją regułami dodawania wobec wszystkich wpisów poza edytowanym) i usunięcie wpisu po `id`, tak żeby komendy tylko zapisywały wynik.

**Contract**:
- `replace_fill_up(start: &StartPoint, fill_ups: &[FillUp], id: u64, new: NewFillUp) -> Result<Vec<FillUp>, EditRejection>` — `EditRejection` = `NotFound` | `Invalid(FillUpRejection)`. Walidacja: `validate_fill_up(start, <wpisy z id != id>, &new)`. Wynik: ta sama lista, wpis `id` zastąpiony przez `new.with_id(id)` na swojej pozycji (id zachowane, także przy zmianie paliwa).
- `remove_fill_up(fill_ups: &[FillUp], id: u64) -> Option<Vec<FillUp>>` — `None`, gdy nie ma wpisu o tym `id`; bez walidacji (reguły są parami, usunięcie nie łamie spójności).
- Żadna z funkcji nie dotyka `next_fill_up_id` (leży w `VaultData`).
- Nazwy domenowe inne niż nazwy komend Tauri (`update_fill_up`/`delete_fill_up`), żeby `commands.rs` importował je bez kolizji.

#### 2. Testy

**File**: `src-tauri/src/fuel/tests.rs`

**Intent**: Udowodnić przeliczenie po poprawce i usunięciu, w tym ryzyko z roadmapy („ten sam wynik, co wpisanie danych od nowa”).

**Contract** (nowe testy):
- `edit_and_delete_equal_entering_from_scratch` — dla każdego `id` próbki wyroczni: (a) usunięcie, (b) przełączenie `full_tank`, (c) litry +100 cl; lista wynikowa odbudowana od zera przez dodawanie wpisów w kolejności `(odometer_km, id)` przez `validate_fill_up` (każdy przyjęty) daje identyczne `fuel_reports` i identyczny wynik `compute` dla każdego `id`.
- `delete_full_tank_merges_top_ups_into_next_period` — próbka bez wpisu 4: wynik wpisu 6 = 6 635 cl / 950 km (6,9842… l/100 km), koszt okresu 20 337 gr; średnia LPG 14 835 cl / 2 010 km, `periods = 3`; benzyna bez zmian (5 277 cl / 2 300 km, `periods = 2`).
- `delete_only_full_tank_gives_no_closed_period` — paliwo z jedną dolewką i jednym pełnym: po usunięciu pełnego `NoClosedPeriod`; po usunięciu ostatniego wpisu paliwa `NoEntries`.
- `edit_unchanged_entry_is_accepted` — każdy wpis próbki zapisany bez zmian jest przyjęty, raporty identyczne.
- `edit_fuel_moves_entry_and_recomputes_both` — wpis 10 benzyna → gaz: wynik wpisu 10 = 777 cl / 290 km (2,6793… l/100 km), `latest` LPG = wpis 10, średnia LPG 18 112 cl / 2 300 km, `periods = 5`, historia LPG `[10, 9, 8, 6, 4, 3, 1]`; benzyna `latest` = wpis 5, średnia 4 000 cl / 900 km, `periods = 1`, historia `[7, 5, 2]`; `id` 10 zachowane.
- `edit_rejected_reports_reason` — wpis 6 z przebiegiem 100 900 (gaz) → `Invalid(SameFuelSameOdometer { date: "2026-01-12" })`.
- `edit_or_delete_unknown_id_is_not_found` — `id` 99: `NotFound` / `None`.

### Success Criteria:

#### Automated Verification:

- Testy domeny przechodzą: `cargo test fuel::tests` w `src-tauri/`
- Cały zestaw testów przechodzi: `cargo test` w `src-tauri/`
- Brak ostrzeżeń kompilacji o martwym kodzie: `cargo check` w `src-tauri/`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 2: Kontrakt — `update_fill_up` i `delete_fill_up`

### Overview

Dwie nowe komendy Tauri według wzorca `add_fill_up`, testy kształtu JSON i trwałości na sejfie w `TempDir`, rejestr kontraktu i typy TS.

### Changes Required:

#### 1. Komendy i DTO

**File**: `src-tauri/src/fuel/commands.rs`

**Intent**: Wystawić poprawkę i usunięcie wpisu przez `with_vault` i `commit`, zwracając wynik poprawionego wpisu z jednego `compute`.

**Contract** (camelCase, unie z `kind`):
- `update_fill_up(id: u64, input: FillUpInput) -> UpdateFillUpResult`: `saved` (+ `result: FillUpResult` poprawionego wpisu), `notFound`, `noStartPoint`, `rejected` (+ `reason: FillUpRejection`, istniejące warianty).
- `delete_fill_up(id: u64) -> DeleteFillUpResult`: `deleted`, `notFound`.
- `apply_update` / `apply_delete` (wzorzec `apply_fill_up`): odrzucenie i `notFound` nie wywołują `commit` (ani pamięć, ani dysk się nie zmieniają); `next_fill_up_id` bez zmian w obu.
- Testy: kształt JSON wszystkich wariantów obu wyników; na sejfie w `TempDir` (tanie parametry KDF, istniejący `new_vault`): poprawiony wpis widać w `overview` i po ponownym otwarciu z dysku, `id` zachowane; odrzucona poprawka i nieznane `id` zostawiają bajty `kmplus.vault` i dane sesji bez zmian; usunięty wpis znika także po ponownym otwarciu, a kolejne `add_fill_up` dostaje nowe `id` (usunięte nie jest używane ponownie).

#### 2. Rejestracja

**File**: `src-tauri/src/lib.rs`

**Intent**: Dodać obie komendy do importu i do `generate_handler!`, inaczej wywołanie z frontendu zawiedzie w czasie działania.

**Contract**: `generate_handler![…, update_fill_up, delete_fill_up]`.

#### 3. Rejestr kontraktu

**File**: `docs/reference/contract-surfaces.md`

**Intent**: Dopisać wiersze `update_fill_up` i `delete_fill_up` do tabeli komend; zaznaczyć, że `FillUpResult` i `FillUpRejection` są wspólne dla `add_fill_up` i `update_fill_up`, a obie komendy nie zmieniają formatu sejfu.

**Contract**: tabela „Tauri commands” i lista „Typy zagnieżdżone”.

#### 4. Typy i wrappery frontendu

**File**: `src/fuel/api.ts`

**Intent**: Odbić `UpdateFillUpResult` i `DeleteFillUpResult` 1:1 z DTO i dodać wrappery `updateFillUp(id, input)` / `deleteFillUp(id)`.

**Contract**: `invoke("update_fill_up", { id, input })`, `invoke("delete_fill_up", { id })`; `FillUpInput`, `FillUpResult`, `FillUpRejection` używane ponownie.

### Success Criteria:

#### Automated Verification:

- Testy kształtu JSON i komend przechodzą: `cargo test fuel::commands` w `src-tauri/`
- Cały zestaw testów przechodzi: `cargo test` w `src-tauri/`
- Frontend kompiluje się z nowymi typami: `npm run build`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 3: UI — tryb edycji, akcje wiersza, panel statusu

### Overview

Wspólny formularz z trybem edycji, „Edytuj”/„Usuń” z potwierdzeniem w wierszach historii, jeden panel statusu ostatniej akcji. Stan wspólny w nowym hooku, `AppShell` zmieniony minimalnie.

### Changes Required:

#### 1. Wspólny stan edycji

**File**: `src/fuel/useFillUpEditing.ts` (nowy)

**Intent**: Jedno miejsce na stan dzielony przez formularz i obie listy, żeby `AppShell` tylko wywołał hook i przekazał propsy.

**Contract**: stan `editing: { fuel: Fuel; row: HistoryRow } | null`, `confirmingId: number | null` (jeden wiersz naraz na cały ekran), `notice: Notice | null`, gdzie `Notice` = `{ kind: "added" | "updated"; fuel; date; result: FillUpResult }` | `{ kind: "deleted"; fuel; date; odometerKm }`; akcje: rozpoczęcie edycji (czyści `notice` i `confirmingId`), anulowanie, zgłoszenie zapisu/poprawki/usunięcia (ustawia `notice`, a usunięcie edytowanego wpisu kończy edycję).

#### 2. Formularz w trybie edycji

**File**: `src/fuel/FillUpForm.tsx`, `src/fuel/format.ts`

**Intent**: Ten sam formularz obsługuje dodawanie i poprawkę: w trybie edycji pola są wypełnione wartościami wpisu, nagłówek „Popraw tankowanie”, przyciski „Zapisz zmiany” i „Anuluj”, wywołanie `updateFillUp`; parsowanie, strażniki u32 i `rejectionMessage` wspólne. Po zapisie lub anulowaniu powrót do trybu dodawania (paliwo zostaje, data = dziś, pozostałe pola puste). `notFound` przy poprawce: wyjście z edycji, odświeżenie widoku i komunikat błędu „Ten wpis już nie istnieje.” (ten sam tekst co w rejestrze serwisów S-05).

**Contract**: propsy formularza rozszerzone o `editing`, `notice` i callbacki hooka; lokalny stan `saved` zastąpiony przez `notice`. W `format.ts` formatery odwrotne do parserów dla wypełnienia pól: centylitry/grosze → tekst z przecinkiem (`4635` → `"46,35"`, round-trip z `parseDecimal2`), km → same cyfry.

#### 3. Panel statusu

**File**: `src/fuel/FillUpResultPanel.tsx`

**Intent**: Panel renderuje `Notice`: „Zapisano: <paliwo>, <data>” lub „Poprawiono: <paliwo>, <data>” z wynikiem (dotychczasowy układ spalania/dolewki), „Usunięto: <paliwo>, <data>, <przebieg>” bez liczb.

**Contract**: props `{ notice: Notice }`; wyłącznie formatery z `format.ts`.

#### 4. Akcje wiersza historii

**File**: `src/fuel/FillUpHistory.tsx`

**Intent**: Każdy wiersz ma „Edytuj” i „Usuń”; „Usuń” przełącza wiersz w potwierdzenie „Usunąć tankowanie z <data>, <przebieg>? [Usuń] [Anuluj]”; potwierdzone wywołuje `deleteFillUp`, zgłasza `deleted` do hooka i odświeża widok; `notFound` też odświeża. Wiersz w edycji jest wyróżniony; przyciski nieaktywne w trakcie żądania. Wzorzec potwierdzenia „Usunąć <opis wpisu>? [Usuń] [Anuluj]” jest wspólny z rejestrem serwisów S-05.

**Contract**: props rozszerzone o `editingId`, `confirmingId` i callbacki hooka; kolejność wierszy dalej z backendu.

#### 5. Powłoka i style

**File**: `src/AppShell.tsx`, `src/App.css`

**Intent**: `AppShell` wywołuje hook i przekazuje jego stan/callbacki do `FillUpForm` i obu `FillUpHistory` (plus `fetchOverview` po każdej akcji) — bez zmiany struktury bloku, który S-05 owija w zakładkę „Paliwo”. Style akcji wiersza, potwierdzenia i wyróżnienia edytowanego wiersza, także w trybie ciemnym i przy wąskim oknie.

**Contract**: brak nowych elementów układu w `AppShell` poza propsami.

### Success Criteria:

#### Automated Verification:

- Frontend kompiluje się bez nieużywanych importów: `npm run build`
- Testy backendu nadal przechodzą: `cargo test` w `src-tauri/`

#### Manual Verification:

- Przed testem prawdziwe pliki z `%APPDATA%\io.github.m-swietek.kmplus\` przeniesione na bok z zanotowanym SHA-256 `kmplus.vault`; `npm run tauri dev` na pustym katalogu; scenariusz bazowy z S-03 wpisany: start 2026-01-01, 100 000 km; A gaz dolewka 2026-01-03, 100 200 km, 20 l, 60 zł; B gaz pełny 2026-01-05, 100 400 km, 20 l, 60 zł; C benzyna pełny 2026-01-06, 100 500 km, 5 l, 32,50 zł; D gaz pełny 2026-01-10, 100 900 km, 45 l, 135 zł — karta Gaz 9,00 l/100 km, średnio 9,44 l/100 km na 900 km
- „Edytuj” przy B wypełnia formularz (Gaz, 2026-01-05, 100400, 20,00, 60,00, do pełna) z nagłówkiem „Popraw tankowanie”; „Anuluj” wraca do „Dodaj tankowanie” bez zmian w danych; „Zapisz zmiany” bez zmian pól jest przyjęte, panel „Poprawiono: Gaz, 05.01.2026” z 10,00 l/100 km na 400 km
- Poprawka B na 30 l: wiersz B 12,50 l/100 km na 400 km, koszt okresu 120,00 zł, cena 2,00 zł/l; karta Gaz 9,00 l/100 km, średnio 10,56 l/100 km na 900 km
- Poprawka A z gazu na benzynę: A przechodzi do listy Benzyna; karta Benzyna 5,00 l/100 km, średnio 5,00 l/100 km na 500 km (wiersz C: koszt okresu 92,50 zł); karta Gaz 9,00 l/100 km, średnio 8,33 l/100 km na 900 km (wiersz B: 7,50 l/100 km, koszt okresu 60,00 zł)
- Poprawka C z przebiegiem 100 950 km jest odrzucona komunikatem „Wpis koliduje z tankowaniem z 10.01.2026 przy 100 900 km (przebieg i data muszą rosnąć razem).”, formularz zostaje w trybie edycji, dane bez zmian
- „Usuń” przy B pokazuje „Usunąć tankowanie z 05.01.2026, 100 400 km?”; „Anuluj” zostawia wiersz; potwierdzone usunięcie: karta Gaz 5,00 l/100 km, średnio 5,00 l/100 km na 900 km, wiersz D koszt okresu 135,00 zł, panel „Usunięto: Gaz, 05.01.2026, 100 400 km”
- „Edytuj” przy D, potem potwierdzone „Usuń” przy D: formularz wraca do trybu dodawania, karta Gaz „Brak wpisów”, lista Gaz „Brak tankowań”, panel „Usunięto: Gaz, 10.01.2026, 100 900 km”
- Po restarcie i odblokowaniu widok identyczny; tryb ciemny i wąskie okno czytelne; po teście prawdziwe pliki przywrócone, SHA-256 `kmplus.vault` równy zanotowanemu przed testem

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Phase 4: Wydanie 0.1.7

### Overview

Wydanie istniejącym pipeline'em według rytuału z S-02/S-03 i `context/deployment/deploy-plan.md`: start workflow tylko po wyraźnym „start” użytkownika, publikacja draftu wyłącznie ręką użytkownika. Numer: 0.1.7, a jeśli S-05 zostanie wydane wcześniej — kolejny numer po wersji S-05. Format sejfu się nie zmienia w S-04, więc dowodem jest aktualizacja bez zmiany pliku przy odblokowaniu; potem poprawka i usunięcie sprawdzone na prawdziwym dzienniku na wpisie testowym, żeby nie naruszać prawdziwej historii.

### Changes Required:

#### 1. Wersja

**File**: `src-tauri/tauri.conf.json`, `package.json`, `src-tauri/Cargo.toml` (+ `src-tauri/Cargo.lock`, `package-lock.json`)

**Intent**: Podbić wersję we wszystkich miejscach, commit, push na `main`, uruchomić `release.yml` po „start” użytkownika i zweryfikować draft.

**Contract**: wersje zgodne; tag `app-v0.1.7` (lub kolejny po S-05).

#### 2. Zapis przebiegu

**File**: `context/changes/correct-or-delete-fillup/change.md` (`## Notes`), `context/deployment/deploy-plan.md` (`## Przebieg wykonania`)

**Intent**: Ślad dowodowy: numer runu z zielonym krokiem „Rust tests”, SHA-256 `kmplus.vault` przed i po aktualizacji z odblokowaniem bez zapisu, wynik sprawdzenia na prawdziwych danych.

**Contract**: dopisane wiersze; brak zmian w kodzie.

### Success Criteria:

#### Automated Verification:

- Workflow wydania kończy się sukcesem, krok „Rust tests” zielony: `gh run view <id>` na runie `release.yml`
- Draft wydania zawiera instalator, `.sig` i `latest.json`: `gh release view app-v<wersja>`

#### Manual Verification:

- Przed aktualizacją: katalog danych zawiera prawdziwy sejf (rozmiar i SHA-256 zgodne z ostatnim zapisanym dowodem — notatką z Fazy 3 albo, jeśli w międzyczasie wydano S-05, dowodem z jego wydania w `deploy-plan.md`), SHA-256 `kmplus.vault` zanotowany na zainstalowanej wersji
- Aktualizacja przez baner; to samo hasło odblokowuje; SHA-256 `kmplus.vault` po odblokowaniu bez zapisu identyczny jak przed aktualizacją; nie powstaje nowy plik `pre-v*`
- Na prawdziwych danych: zanotowane wartości kart; dodana dolewka testowa (dzisiejsza data, przebieg powyżej ostatniego znanego) poprawiona przez „Edytuj” (panel „Poprawiono…”), potem usunięta z potwierdzeniem (panel „Usunięto…”); karty i historie wracają do zanotowanych wartości; przebieg zapisany w `change.md` i `deploy-plan.md`

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human that the manual testing was successful before proceeding to the next phase.

---

## Testing Strategy

### Unit Tests:

- `cargo test` (Faza 1): równoważność „poprawka/usunięcie = wpisanie od nowa” dla każdego wpisu próbki wyroczni (usunięcie, przełączenie `full_tank`, zmiana litrów); usunięcie pełnego scala dolewki z następnym okresem; usunięcie jedynego pełnego → `NoClosedPeriod`, ostatniego wpisu → `NoEntries`; zapis bez zmian przyjęty; zmiana paliwa przelicza oba paliwa; odrzucenie z powodem; nieznane `id`.
- Testy kształtu JSON (Faza 2) dla wszystkich wariantów `UpdateFillUpResult` i `DeleteFillUpResult`.

### Integration Tests:

- `apply_update` / `apply_delete` na sejfie w `TempDir` (Faza 2): trwałość po ponownym otwarciu, zachowane `id`, brak ponownego użycia `id` po usunięciu, odrzucenie i `notFound` bez zmiany bajtów pliku.

### Manual Testing Steps:

1. Scenariusz z Fazy 3 krok po kroku na pustym katalogu danych (prawdziwe pliki odłożone i przywrócone z weryfikacją SHA-256).
2. Restart i odblokowanie — widok bez zmian.
3. Aktualizacja do nowej wersji na prawdziwych danych i sprawdzenie na wpisie testowym (Faza 4).

## Performance Considerations

Poprawka i usunięcie to jedno `compute` (wynik poprawionego wpisu) plus istniejące `fuel_overview` po odświeżeniu; liniowe w liczbie wpisów, bez znaczenia przy dzienniku jednego auta. Zapis pliku jak przy `add_fill_up` (świadomie pominięte F4 z S-02).

## Migration Notes

Brak zmiany formatu sejfu (S-04 nie dodaje pól; obowiązuje format bieżący w chwili scalenia — v2, albo v3, jeśli S-05 wejdzie wcześniej), brak migratora i nowej kopii `pre-v*`. Nowe są tylko komendy `update_fill_up` i `delete_fill_up`, wydawane razem z frontendem.

Ryzyka scalenia z równoległym S-05 (`plan-service`): wspólne pliki `src/AppShell.tsx` (zakładki „Paliwo / Serwisy” — S-04 ogranicza się do wywołania hooka i propsów; konflikt pewny, rozwiązanie: struktura zakładek z S-05 + propsy z S-04), `src-tauri/src/lib.rs` (`generate_handler!`), `docs/reference/contract-surfaces.md` (S-05 dodaje komendy i pola ładunku v3) oraz pliki wersji. `src/App.css` zmienia tylko S-04 (S-05 trzyma style w `src/service/service.css`). Jeśli S-05 zostanie wydane pierwsze, numer wydania S-04 to kolejna wersja po nim. Kolejność faz ręcznych i wydań — patrz Critical Implementation Details.

## References

- Roadmap: `context/foundation/roadmap.md` (S-04)
- PRD: `context/foundation/prd.md` (US-01, FR-004, FR-005)
- Poprzednia zmiana: `context/archive/2026-10-26-fillup-history-both-fuels/plan.md`, `research.md`, `reviews/impl-review.md` (F2 — higiena katalogu danych)
- Wcześniejsze decyzje: `context/archive/2026-10-11-first-fillup-consumption/plan.md:42,54` (`id` wpisu dla S-04, wyniki liczone od zera)
- Reguła i walidacja: `src-tauri/src/fuel/mod.rs:128-225,265-322`
- Wzorzec komendy: `src-tauri/src/fuel/commands.rs:310-391`
- Zapis sejfu: `src-tauri/src/vault/mod.rs:118-170`
- UI: `src/AppShell.tsx:64-86`, `src/fuel/FillUpForm.tsx`, `src/fuel/FillUpHistory.tsx`, `src/fuel/FillUpResultPanel.tsx`

## Progress

> Convention: `- [ ]` pending, `- [x]` done. Append ` — <commit sha>` when a step lands. Do not rename step titles. See `references/progress-format.md`.

### Phase 1: Domena — poprawka i usunięcie wpisu

#### Automated

- [x] 1.1 Testy domeny przechodzą: `cargo test fuel::tests` w `src-tauri/`
- [x] 1.2 Cały zestaw testów przechodzi: `cargo test` w `src-tauri/`
- [x] 1.3 Brak ostrzeżeń kompilacji o martwym kodzie: `cargo check` w `src-tauri/`

### Phase 2: Kontrakt — `update_fill_up` i `delete_fill_up`

#### Automated

- [ ] 2.1 Testy kształtu JSON i komend przechodzą: `cargo test fuel::commands` w `src-tauri/`
- [ ] 2.2 Cały zestaw testów przechodzi: `cargo test` w `src-tauri/`
- [ ] 2.3 Frontend kompiluje się z nowymi typami: `npm run build`

### Phase 3: UI — tryb edycji, akcje wiersza, panel statusu

#### Automated

- [ ] 3.1 Frontend kompiluje się bez nieużywanych importów: `npm run build`
- [ ] 3.2 Testy backendu nadal przechodzą: `cargo test` w `src-tauri/`

#### Manual

- [ ] 3.3 Przed testem prawdziwe pliki z `%APPDATA%\io.github.m-swietek.kmplus\` przeniesione na bok z zanotowanym SHA-256 `kmplus.vault`; `npm run tauri dev` na pustym katalogu; scenariusz bazowy z S-03 wpisany: start 2026-01-01, 100 000 km; A gaz dolewka 2026-01-03, 100 200 km, 20 l, 60 zł; B gaz pełny 2026-01-05, 100 400 km, 20 l, 60 zł; C benzyna pełny 2026-01-06, 100 500 km, 5 l, 32,50 zł; D gaz pełny 2026-01-10, 100 900 km, 45 l, 135 zł — karta Gaz 9,00 l/100 km, średnio 9,44 l/100 km na 900 km
- [ ] 3.4 „Edytuj” przy B wypełnia formularz (Gaz, 2026-01-05, 100400, 20,00, 60,00, do pełna) z nagłówkiem „Popraw tankowanie”; „Anuluj” wraca do „Dodaj tankowanie” bez zmian w danych; „Zapisz zmiany” bez zmian pól jest przyjęte, panel „Poprawiono: Gaz, 05.01.2026” z 10,00 l/100 km na 400 km
- [ ] 3.5 Poprawka B na 30 l: wiersz B 12,50 l/100 km na 400 km, koszt okresu 120,00 zł, cena 2,00 zł/l; karta Gaz 9,00 l/100 km, średnio 10,56 l/100 km na 900 km
- [ ] 3.6 Poprawka A z gazu na benzynę: A przechodzi do listy Benzyna; karta Benzyna 5,00 l/100 km, średnio 5,00 l/100 km na 500 km (wiersz C: koszt okresu 92,50 zł); karta Gaz 9,00 l/100 km, średnio 8,33 l/100 km na 900 km (wiersz B: 7,50 l/100 km, koszt okresu 60,00 zł)
- [ ] 3.7 Poprawka C z przebiegiem 100 950 km jest odrzucona komunikatem „Wpis koliduje z tankowaniem z 10.01.2026 przy 100 900 km (przebieg i data muszą rosnąć razem).”, formularz zostaje w trybie edycji, dane bez zmian
- [ ] 3.8 „Usuń” przy B pokazuje „Usunąć tankowanie z 05.01.2026, 100 400 km?”; „Anuluj” zostawia wiersz; potwierdzone usunięcie: karta Gaz 5,00 l/100 km, średnio 5,00 l/100 km na 900 km, wiersz D koszt okresu 135,00 zł, panel „Usunięto: Gaz, 05.01.2026, 100 400 km”
- [ ] 3.9 „Edytuj” przy D, potem potwierdzone „Usuń” przy D: formularz wraca do trybu dodawania, karta Gaz „Brak wpisów”, lista Gaz „Brak tankowań”, panel „Usunięto: Gaz, 10.01.2026, 100 900 km”
- [ ] 3.10 Po restarcie i odblokowaniu widok identyczny; tryb ciemny i wąskie okno czytelne; po teście prawdziwe pliki przywrócone, SHA-256 `kmplus.vault` równy zanotowanemu przed testem

### Phase 4: Wydanie 0.1.7

#### Automated

- [ ] 4.1 Workflow wydania kończy się sukcesem, krok „Rust tests” zielony: `gh run view <id>` na runie `release.yml`
- [ ] 4.2 Draft wydania zawiera instalator, `.sig` i `latest.json`: `gh release view app-v<wersja>`

#### Manual

- [ ] 4.3 Przed aktualizacją: katalog danych zawiera prawdziwy sejf (rozmiar i SHA-256 zgodne z ostatnim zapisanym dowodem — notatką z Fazy 3 albo, jeśli w międzyczasie wydano S-05, dowodem z jego wydania w `deploy-plan.md`), SHA-256 `kmplus.vault` zanotowany na zainstalowanej wersji
- [ ] 4.4 Aktualizacja przez baner; to samo hasło odblokowuje; SHA-256 `kmplus.vault` po odblokowaniu bez zapisu identyczny jak przed aktualizacją; nie powstaje nowy plik `pre-v*`
- [ ] 4.5 Na prawdziwych danych: zanotowane wartości kart; dodana dolewka testowa (dzisiejsza data, przebieg powyżej ostatniego znanego) poprawiona przez „Edytuj” (panel „Poprawiono…”), potem usunięta z potwierdzeniem (panel „Usunięto…”); karty i historie wracają do zanotowanych wartości; przebieg zapisany w `change.md` i `deploy-plan.md`
