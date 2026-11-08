---
date: 2026-10-26T20:40:48+01:00
researcher: Claude (Opus 5.5) for m-swietek
git_commit: b59933e033fc7f11517f36a7d7a23daf73a7b3e7
branch: main
repository: 10xdevs4 (km-plus)
topic: "S-03 fillup-history-both-fuels — co kod i decyzje z S-02 już dają, a czego brakuje do historii tankowań i bieżącego spalania obu paliw"
tags: [research, codebase, fuel, consumption, fuel_overview, AppShell, vault-format]
status: complete
last_updated: 2026-10-26
last_updated_by: Claude (Opus 5.5)
---

# Research: S-03 fillup-history-both-fuels

**Date**: 2026-10-26T20:40:48+01:00
**Researcher**: Claude (Opus 5.5) for m-swietek
**Git Commit**: b59933e
**Branch**: main
**Repository**: 10xdevs4 (km-plus)

## Research Question

Roadmapa S-03 (`context/foundation/roadmap.md:109-120`): użytkownik może przeglądać historię tankowań oraz bieżące spalanie osobno dla gazu i benzyny, także przy przeplatanych tankowaniach obu paliw i częściowych dolewkach (US-01, FR-007). Co z tego istnieje w kodzie po S-02, jakie decyzje S-03 dziedziczy, czego brakuje i co zostaje do decyzji użytkownika?

## Summary

- **Reguła spalania dla przeplatanych paliw i dolewek już jest zaimplementowana i przetestowana.** `compute` (`src-tauri/src/fuel/mod.rs:183-224`) liczy każde paliwo osobno, sortuje wpisy danego paliwa po przebiegu (`mod.rs:189`), sumuje litry i kwoty dolewek do najbliższego tankowania do pełna (`mod.rs:200`). Zwraca wynik dla każdego wpisu listy. Przeplatanie i dolewki pokrywają testy `oracle_sample_matches_hand_computed_results`, `fuels_are_paired_only_with_same_fuel`, `top_up_litres_count_toward_next_full_period` (`src-tauri/src/fuel/tests.rs:113,205,190`). S-03 nie musi zmieniać reguły — ryzyko z roadmapy dotyczy tego, czy widok historii pokaże wyniki tej samej funkcji, a nie nowego liczenia.
- **„Bieżące spalanie” per paliwo jest już widoczne** w kartach paliw (`src/AppShell.tsx:62-66,76-91`) przez komendę `fuel_overview` (`src-tauri/src/fuel/commands.rs:179`). Definicja z S-02: wynik tankowania do pełna danego paliwa o najwyższym przebiegu (`mod.rs:228-243`).
- **Brakuje historii.** Na tym commicie (inspekcja `lib.rs:15-23`, `commands.rs`) żadna zarejestrowana komenda nie zwraca listy tankowań ani wyników per wpis; frontend nie ma widoku listy (`src/fuel/` zawiera 6 plików, żaden nie renderuje listy). S-02 jawnie odłożył listę do S-03 (`context/archive/2026-10-11-first-fillup-consumption/plan.md:41`).
- **S-03 prawdopodobnie nie wymaga zmiany formatu sejfu.** Wyniki nie są zapisywane (`docs/reference/contract-surfaces.md:40`), a `FillUp` ma już `id` (`mod.rs:43`). Komenda tylko do odczytu, która woła `compute` na `VaultData`, nie dotyka ładunku v2 (`src-tauri/src/vault/data.rs:12`).
- **Do decyzji użytkownika** (nierozstrzygnięte w PRD ani S-02): układ historii (jedna lista przeplatana czy per paliwo, kierunek sortowania, kolumny) i komunikat „brak wyniku” — w tym, czy rozróżniać „brak wpisów”, „same dolewki” i „otwarty okres po ostatnim pełnym”.

## Detailed Findings

### Domena: reguła spalania (`src-tauri/src/fuel/mod.rs`)

- `compute(start, fill_ups) -> Vec<(u64, FillUpResult)>` (`mod.rs:183`) zwraca jedną parę `(id, wynik)` na każdy wpis wejścia, w kolejności wejścia (asercja kolejności: `tests.rs:136-138`). Dla każdego z 2 paliw (`mod.rs:185`) wpisy tego paliwa są sortowane po `(odometer_km, id)` (`mod.rs:189`), pierwszą kotwicą jest punkt startowy (`mod.rs:191`).
- Wpis daje `Consumption` tylko gdy `full_tank && distance_km > 0` (`mod.rs:200`); w innym przypadku `TopUp { price_per_litre_grosze }`. Po wyniku sumy się zerują, kotwica przesuwa się na ten wpis (`mod.rs:209-211`).
- `latest_consumption(start, fill_ups, fuel)` (`mod.rs:228`) wybiera tankowanie do pełna danego paliwa z najwyższym `(odometer_km, id)` (`mod.rs:236`) i zwraca jego wynik z `compute`. Wywołuje `compute` dla całej listy przy każdym wywołaniu — `overview` robi to 2 razy (raz na paliwo, `commands.rs:246-247`). Przy jednoosobowym dzienniku to nie jest problem wydajności; tylko obserwacja.
- `validate_fill_up` (`mod.rs:128-174`) odrzuca wpis, jeśli przebieg i data nie rosną razem z którymkolwiek istniejącym wpisem (`OrderConflict`), a ten sam przebieg dopuszcza wyłącznie dla innego paliwa z tą samą datą (`mod.rs:169-175`). Wniosek dla sortowania historii: dla danych przyjętych przez tę walidację porządek po przebiegu i porządek po dacie są zgodne; remis występuje tylko dla pary różnych paliw na tym samym przebiegu i dacie (przykład: wiersze 4/5 próbki wyroczni, `tests.rs:100-101`). Potrzebny jest deterministyczny tie-break (np. `id` albo paliwo).

### Komendy Tauri (`src-tauri/src/fuel/commands.rs`, `src-tauri/src/lib.rs`)

- Zarejestrowane komendy (`lib.rs:15-23`): `vault_status`, `vault_setup`, `vault_unlock`, `vault_restore_backup`, `fuel_overview`, `set_start_point`, `add_fill_up` — 7 komend, żadna nie zwraca listy wpisów.
- `FuelOverview { start_point, lpg, petrol }` (`commands.rs:24-28`); `lpg`/`petrol` to `Option<LatestConsumption>` (`commands.rs:40-48`). `overview` mapuje `TopUp` na `None` (`commands.rs:238`) — „tylko zamknięty okres daje liczbę”.
- `None` w karcie nie rozróżnia „brak wpisów tego paliwa” od „są tylko dolewki” (test `latest_consumption_none_with_only_top_ups`, `tests.rs:284-293`). Jeśli S-03 ma rozróżniać te stany, kontrakt `FuelOverview` (lub nowej komendy) musi przenieść tę informację.
- Wzorce do naśladowania przy nowej komendzie: DTO w `commands.rs`, `kind`-tagowane unie w camelCase (`commands.rs:51-55,78-111`), testy kształtu JSON (`commands.rs:297-600`), helper `with_vault` z błędem „dane są zablokowane; odblokuj aplikację” (`commands.rs:20,205-212`). Nowa komenda musi trafić do `generate_handler!` (`lib.rs:15`) i do tabeli w `docs/reference/contract-surfaces.md:54-62`.
- Domenowe `FillUpResult` jest `#[derive(Debug, Clone, PartialEq)]` bez serde (`mod.rs:98-99`); mapowanie na widok istnieje już jako `From<FillUpResult> for FillUpResultView` (`commands.rs:139-162`) — do ponownego użycia w wierszach historii.

### Frontend (`src/`)

- `AppShell` (`src/AppShell.tsx`) po punkcie startowym renderuje 2 karty paliw i formularz (`AppShell.tsx:56-71`). Po zapisie formularz woła `onSaved={fetchOverview}` (`AppShell.tsx:69`), czyli odświeża tylko overview; historia musiałaby być odświeżana tym samym hookiem, żeby spełnić US-01 „widoczne bez dodatkowej akcji”.
- Pusty stan karty: „Brak wyniku — potrzebne tankowanie do pełna” (`AppShell.tsx:88`). Wynik dolewki po zapisie: „Dolewka — brak wyniku spalania (okres zamknie tankowanie do pełna) · Cena: …” (`src/fuel/FillUpResultPanel.tsx:24-27`). To istniejące precedensy dla nierozstrzygniętego komunikatu z roadmapy.
- Typy w `src/fuel/api.ts` odzwierciedlają DTO ręcznie (`api.ts:1-3` wskazuje `contract-surfaces.md` jako rejestr); nowa komenda wymaga nowego typu i wrappera `invoke`. `FillUpResult` (TS, `api.ts:40-49`) da się użyć ponownie dla wierszy.
- Formatowanie (`src/fuel/format.ts`): `formatConsumption`, `formatMoney`, `formatPricePerLitre`, `formatKm`, `formatDate` — komplet potrzebny do wiersza historii; `pl-PL`, 2 miejsca po przecinku tylko przy wyświetlaniu.
- Brak runnera testów frontendu; jedyną bramką jest `npm run build` (tsc strict, AGENTS.md). Weryfikacja UI to ręczne scenariusze.

### Format danych (`src-tauri/src/vault/data.rs`)

- `CURRENT_FORMAT_VERSION = 2` (`data.rs:12`); `VaultData` zawiera `start_point`, `fill_ups: Vec<FillUp>`, `next_fill_up_id` (`data.rs:17-28`), bez `#[serde(default)]`. Komenda tylko do odczytu nie zmienia tego ładunku. Jeśli plan doda jakiekolwiek pole (np. zapamiętane preferencje widoku), obowiązuje wzorzec `migrate_v2_to_v3` + kopia przed migracją (`data.rs:1-5`; S-02 `plan.md:367`).

## Code References

- `src-tauri/src/fuel/mod.rs:128-174` — `validate_fill_up`: przebieg i data muszą rosnąć razem.
- `src-tauri/src/fuel/mod.rs:183-224` — `compute`: reguła pełny-do-pełnego per paliwo, wynik na każdy wpis.
- `src-tauri/src/fuel/mod.rs:228-243` — `latest_consumption`: definicja „bieżącego spalania”.
- `src-tauri/src/fuel/commands.rs:24-48` — `FuelOverview`, `LatestConsumption`.
- `src-tauri/src/fuel/commands.rs:102-170` — widoki odrzuceń/wyników i konwersje `From`.
- `src-tauri/src/fuel/commands.rs:177-212` — komendy i `with_vault`.
- `src-tauri/src/fuel/commands.rs:214-249` — `overview`: `TopUp` → `None`.
- `src-tauri/src/fuel/tests.rs:96-164` — próbka wyroczni (10 wpisów, przeplatane paliwa i dolewki) z ręcznie policzonymi wynikami.
- `src-tauri/src/lib.rs:15-23` — `generate_handler!`.
- `src/AppShell.tsx:58-91` — karty paliw, pusty stan, odświeżanie po zapisie.
- `src/fuel/FillUpResultPanel.tsx:13-29` — prezentacja wyniku i dolewki.
- `src/fuel/api.ts` — typy kontraktu i wrappery `invoke`.
- `src-tauri/src/vault/data.rs:12-28` — format v2.
- `docs/reference/contract-surfaces.md:33-62` — rejestr ładunku i komend.

## Architecture Insights

- Wyniki są czystą funkcją `(start_point, fill_ups)`, liczoną od zera przy każdym odczycie (`mod.rs:1-6`). Dzięki temu historia, karta i panel po zapisie pokażą te same liczby, jeśli wszystkie korzystają z `compute` — i S-04 (edycja/usuwanie) dostanie przeliczenie za darmo. Historia nie powinna mieć własnej logiki parowania po stronie frontendu.
- Granica kontraktu: domena bez serde, DTO i kształt JSON w `commands.rs`, rejestr w `contract-surfaces.md`, ręczne odbicie typów w `api.ts`. Nowa komenda dotyka wszystkich 4 miejsc.
- 3 komendy paliwowe są `async` (`commands.rs:178-202`); 2 z nich (`set_start_point`, `add_fill_up`) robią blokujący zapis pliku przez `vault.commit` (`commands.rs:265,290`). Recenzja S-02 (F4) świadomie to pominęła (S-02 `reviews/impl-review.md:56-64`). `fuel_overview` czyta tylko dane sesji w pamięci (`commands.rs:180`); komenda historii zbudowana tak samo nie robi I/O, więc F4 jej nie dotyczy.

## Historical Context (from prior changes)

- `context/archive/2026-10-11-first-fillup-consumption/plan.md:41` — „Historia tankowań / lista wpisów — S-03.” (odłożone do S-03; potwierdzone też w `plan-brief.md:27,43`).
- `.../plan.md:42` — model ma `id` wpisu, „żeby S-04 nie wymagał migracji”; ids nie są ponownie używane (`contract-surfaces.md:41`).
- `.../plan.md:54` — „najnowsze spalanie” = wynik tankowania do pełna o najwyższym przebiegu; wynik liczony od zera bez zapamiętywania. Zgodne z kodem (`mod.rs:228-243`) — supported.
- `.../plan-brief.md:21` — „koszt” = koszt okresu (suma kwot w okresie) + cena za litr wpisu. Zgodne z `FillUpResult::Consumption` (`mod.rs:101-107`) — supported.
- `.../plan.md:56-61`, `plan-brief.md:24` — niezgodny porządek przebiegu/daty jest odrzucany, wpisy wsteczne pomiędzy istniejącymi są dozwolone i przeliczają okresy (test `tests.rs:223-245`) — supported.
- `.../reviews/impl-review.md:74` (F5) — powody odrzuceń ujednolicone do obiektów `{kind}`; nowe DTO powinny trzymać ten kształt.
- `context/archive/2026-10-09-password-unlocked-vault/plan.md:372` sugerował opcjonalne pola z `#[serde(default)]` bez podbijania wersji — **contradicted** przez nowszą decyzję S-02 (`data.rs:14-15`: brak pola w v2 = uszkodzenie). Obowiązuje wersja + migrator.
- Rytuał wydania (S-02 `plan.md:289-299`): podbicie wersji w `tauri.conf.json`, `package.json`, `Cargo.toml` (+ lockfile), tag `app-vX.Y.Z`, start workflow po „start” użytkownika, dowody w `change.md` i `deploy-plan.md`. Bieżąca wersja: 0.1.4 (`package.json:4`, `tauri.conf.json:4`, `Cargo.toml:3`).

## Related Research

- Nie dotyczy — w `context/changes/**` i `context/archive/**` nie ma wcześniejszych plików `research.md` (sprawdzone: archiwa S-01 i S-02 zawierają `change.md`, `plan.md`, `plan-brief.md`, `reviews/`).

## Open Questions

Wszystkie do rozstrzygnięcia przez użytkownika w `/10x-plan`; żadne nie blokuje planowania.

1. **Układ historii** — jedna przeplatana lista obu paliw (z etykietą paliwa) czy osobne listy/zakładki per paliwo? Kierunek sortowania (najnowsze na górze?) i tie-break dla pary różnych paliw na tym samym przebiegu.
2. **Kolumny wiersza** — które pola wpisu (data, przebieg, litry, kwota, do pełna) i które pola wyniku (spalanie, dystans, koszt okresu, cena/l) pokazywać; czy wiersz dolewki pokazuje tylko cenę/l.
3. **Komunikat „brak wyniku”** (niewiadoma z roadmapy, `roadmap.md:118`) — czy rozróżniać „brak wpisów”, „same dolewki” i „otwarty okres po ostatnim pełnym”; czy pokazywać sumy otwartego okresu (litry/kwota — fakty, nie szacunek). Rozróżnienie wymaga rozszerzenia kontraktu, bo `LatestConsumption | null` go nie niesie.
4. **Czy „bieżące spalanie” zostaje definicją z S-02** (ostatni zamknięty okres), czy plan ją zmienia — zmiana musiałaby być świadomą decyzją.
5. **Wydanie** — czy S-03 kończy się wydaniem 0.1.5 według rytuału z S-02 (numer to wniosek, nie zapisana decyzja).
