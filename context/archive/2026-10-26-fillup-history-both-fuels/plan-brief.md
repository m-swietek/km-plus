# Historia tankowań i spalanie obu paliw — Plan Brief

> Full plan: `context/changes/fillup-history-both-fuels/plan.md`
> Research: `context/changes/fillup-history-both-fuels/research.md`

## What & Why

S-03 roadmapy (US-01, FR-007): historia tankowań i bieżące spalanie osobno dla gazu i benzyny, także przy przeplatanych tankowaniach i dolewkach. Przeplatanie i dolewki to scenariusze, w których najłatwiej o błędną liczbę — a błędna liczba jest gorsza niż brak liczby.

## Starting Point

Reguła pełna-do-pełnej per paliwo (`compute`, `src-tauri/src/fuel/mod.rs:183`) jest gotowa i przetestowana; karty paliw pokazują wynik ostatniego zamkniętego okresu przez `fuel_overview`. Brak listy wpisów, a `null` w karcie nie odróżnia „brak wpisów” od „same dolewki”.

## Desired End State

Karta każdego paliwa pokazuje „Brak wpisów”, „Brak wyniku — potrzebne tankowanie do pełna” albo spalanie ostatniego okresu plus „średnio od startu”. Pod formularzem są dwie listy historii (Gaz, Benzyna), od najnowszego, z wynikiem każdego wpisu. Wszystko odświeża się po zapisie i pochodzi z jednego `compute`. Wydane jako 0.1.5.

## Key Decisions Made

| Decision | Choice | Why (1 sentence) | Source |
| --- | --- | --- | --- |
| „Bieżące spalanie” | Ostatni zamknięty okres **i** średnia od startu na karcie | Ostatni okres pokazuje zmianę, średnia daje stabilny obraz bez wykresu. | Plan |
| Definicja średniej | Σ litrów zamkniętych okresów / Σ ich dystansów; dolewki po ostatnim pełnym wyłączone | Otwarty okres nie ma dystansu — wliczenie dałoby błędną liczbę. | Plan |
| Średnia przy 1 okresie | Widoczna od pierwszego zamkniętego okresu (równa ostatniemu) | Stały układ karty. | Plan |
| Układ historii | Osobna lista per paliwo, od najnowszego (przebieg malejąco, tie-break `id`) | Okresy jednego paliwa czytane bez przerw. | Plan |
| Zawartość wiersza | Pola wpisu + cena/l + pełny/dolewka; pełny wiersz także spalanie, dystans, koszt okresu | Każda liczba sprawdzalna z paragonem. | Plan |
| Brak wyniku | Rozróżnione: „Brak wpisów” / „Brak wyniku — potrzebne tankowanie do pełna”; pusta lista „Brak tankowań”; bez sum otwartego okresu | Mówi, co zrobić dalej, i nie pokazuje liczb wyglądających na wynik. | Plan |
| Kontrakt | Rozszerzenie `fuel_overview` zamiast nowej komendy | Jeden spójny obraz danych i istniejące odświeżanie po zapisie. | Plan |
| Format sejfu | Bez zmian (v2), bez migracji | Wyniki są liczone, nie zapisywane. | Research |
| Wydanie | 0.1.5 rytuałem z S-02 | Historia od razu w prawdziwym dzienniku. | Plan |

## Scope

**In scope:** raport domenowy per paliwo (stan, ostatni okres, średnia, historia), rozszerzone `fuel_overview` + testy kształtu + `contract-surfaces.md` + `api.ts`, karty z trzema stanami i średnią, dwie listy historii, wydanie 0.1.5.

**Out of scope:** edycja/usuwanie (S-04), wspólna przeplatana lista, sortowanie rosnące, filtrowanie/stronicowanie, sumy otwartego okresu, wykres, koszt zł/100 km, zmiana formatu sejfu, zmiana reguły i uproszczenia LPG.

## Architecture / Approach

`compute` (bez zmian) → nowa czysta funkcja raportu obu paliw z jednego wywołania (`FuelSummary`: `NoEntries` | `NoClosedPeriod` | `Consumption { latest, average }` + historia malejąco) → DTO w `commands.rs` (`FuelOverview { startPoint, lpg/petrol: { summary, history } | null }`) → typy w `api.ts` → `AppShell` (karty) i nowy `FillUpHistory.tsx` (listy). TS tylko formatuje; `latest_consumption` znika.

## Phases at a Glance

| Phase | What it delivers | Key risk |
| --- | --- | --- |
| 1. Domena | Raport per paliwo, testy na próbce wyroczni (LPG 8,62, benzyna 2,29 l/100 km) | Średnia zliczająca litry otwartego okresu |
| 2. Kontrakt | Nowy kształt `fuel_overview`, testy JSON, rejestr, typy TS | Rozjazd DTO ↔ `api.ts` (brak testów TS) |
| 3. UI | Karty z trzema stanami i średnią, dwie listy historii | Szeroki wiersz w małym oknie; test na prawdziwym sejfie |
| 4. Wydanie 0.1.5 | Release + dowód aktualizacji 0.1.4 → 0.1.5 bez zmiany pliku | Pominięty zapis dowodów (jak F1 w S-02) |

**Prerequisites:** S-02 done (0.1.4 zainstalowana z prawdziwymi danymi); `gh` z dostępem do repo.
**Estimated effort:** ~2–3 sesje w 4 fazach.

## Open Risks & Assumptions

- `npm run tauri dev` używa tego samego katalogu danych co zainstalowana aplikacja — test ręczny wymaga odłożenia prawdziwego `kmplus.vault` na bok i przywrócenia go po teście.
- Zmiana kształtu `fuel_overview` jest łamiąca; bezpieczna tylko dlatego, że frontend i backend wychodzą w jednym buildzie.
- Uproszczenie LPG (rozruch na benzynie) systematycznie zawyża także średnią gazu — świadomie przyjęte.

## Success Criteria (Summary)

- Przy przeplatanych tankowaniach i dolewkach każde paliwo ma własną historię i własne liczby, zgodne z ręcznym przeliczeniem.
- Paliwo bez zamkniętego okresu nigdy nie pokazuje liczby, a komunikat mówi, czego brakuje.
- Po aktualizacji do 0.1.5 istniejące wpisy widać w historii, a plik sejfu nie zmienia się przy odblokowaniu.
