---
project: kmPlus
version: 1
status: draft                    # draft | active | locked
created: 2026-10-03
updated: 2026-10-03
prd_version: 4
main_goal: quality
top_blocker: skills
milestone_id: trusted-fuel-and-service-log
milestone_seq: 1
milestone_status: open           # open | done
---

# Roadmap: kmPlus

> Derived from `context/foundation/prd.md` (v4) + auto-researched codebase baseline.
> Edit-in-place; archive when superseded.
> Slices below are listed in dependency order. The "At a glance" table is the index.

## Milestone

**M-1: Wiarygodny dziennik tankowań i serwisów** — Status: open

- **Intent:** Zastąpić arkusz kalkulacyjny autora: dane chronione hasłem, poprawne spalanie liczone osobno dla gazu i benzyny oraz przypomnienia serwisowe pokazywane przy otwarciu aplikacji — czyli oba główne kryteria sukcesu z PRD.
- **Source materials:** `context/foundation/prd.md` (v4)
- **Done when:** every S-NN below is `done`.
- **Scope anchors:** FR-001…FR-011 (wszystkie konieczne), US-01…US-03.

## Vision recap

Kierowca auta z instalacją gazową nie zna realnego spalania — komputer pokładowy przekłamuje wynik przy dwóch paliwach, więc dane z każdego paragonu trafiają ręcznie do arkusza, a terminy serwisów żyją osobno w kalendarzu. kmPlus to jednoosobowa, w pełni offline'owa aplikacja desktopowa, która ma dać ten sam obraz spalania co arkusz przy mniejszym wysiłku i dołożyć to, czego arkusz nie robi: aktywny nadzór nad przeglądami i wymianami. Poprzeczką jest własny arkusz autora, nie rynek; błędna liczba jest gorsza niż brak liczby, a utrata historii jest awarią krytyczną.

## North star

**S-02: Użytkownik podaje punkt startowy, dodaje pierwsze tankowanie i od razu widzi spalanie tego paliwa oraz koszt** — to dosłownie pierwsze główne kryterium sukcesu z PRD i miejsce, gdzie reguła parowania tankowań do pełna może dać błędną liczbę; przy celu `quality` chcemy ją sprawdzić jak najwcześniej.

> „Gwiazda przewodnia” (north star) oznacza tu najmniejszy kawałek działający od początku do końca, którego dowiezienie udowadnia, że produkt spełnia swoją główną obietnicę — dlatego stoi tak wcześnie, jak pozwalają zależności: wszystko inne ma sens tylko wtedy, gdy to działa.

## At a glance

| ID   | Change ID                   | Outcome (user can …)                                                                 | Prerequisites | PRD refs                      | Status   |
| ---- | --------------------------- | ------------------------------------------------------------------------------------ | ------------- | ----------------------------- | -------- |
| S-01 | password-unlocked-vault     | ustawić hasło przy pierwszym uruchomieniu i odblokowywać nim zaszyfrowane dane        | —             | FR-001                        | ready    |
| S-02 | first-fillup-consumption    | podać punkt startowy, dodać tankowanie i od razu zobaczyć spalanie tego paliwa i koszt | S-01          | US-01, FR-002, FR-003, FR-007 | proposed |
| S-03 | fillup-history-both-fuels   | przeglądać historię tankowań i bieżące spalanie osobno dla gazu i benzyny             | S-02          | US-01, FR-007                 | proposed |
| S-04 | correct-or-delete-fillup    | poprawić lub usunąć wpis tankowania i zobaczyć przeliczone spalanie                   | S-03          | US-01, FR-004, FR-005         | proposed |
| S-05 | plan-service                | zaplanować, edytować i usunąć przegląd lub wymianę z progiem przebiegu i/lub daty     | S-01          | US-02, FR-008, FR-009         | proposed |
| S-06 | standalone-odometer-entry   | dodać samodzielny wpis przebiegu i zobaczyć najnowszy znany przebieg auta             | S-02          | US-03, FR-006                 | proposed |
| S-07 | service-reminder-on-open    | po otwarciu aplikacji od razu zobaczyć przypomnienie o serwisie, którego próg minął   | S-05, S-06    | US-03, FR-010                 | proposed |
| S-08 | complete-recurring-service  | oznaczyć serwis jako wykonany; cykliczny tworzy kolejne wystąpienie od bieżącego stanu | S-07          | US-03, FR-011                 | proposed |

## Streams

Navigation aid — groups items that share a Prerequisites chain. Canonical ordering still lives in the dependency graph below; this table is the proposed reading order across parallel tracks.

| Stream | Theme                     | Chain                                  | Note                                                                                              |
| ------ | ------------------------- | -------------------------------------- | ------------------------------------------------------------------------------------------------- |
| A      | Dane i spalanie           | `S-01` → `S-02` → `S-03` → `S-04`      | Ścieżka poprawności: najpierw ochrona danych, potem reguła spalania i jej przeliczanie.           |
| B      | Serwisy i przypomnienia   | `S-05` → `S-06` → `S-07` → `S-08`      | `S-05` dołącza do A przy `S-01`, `S-06` przy `S-02`; `S-05` może iść równolegle z gwiazdą `S-02`. |

## Baseline

What's already in place in the codebase as of `2026-10-03` (auto-researched + user-confirmed).
Slices below assume these are present and do NOT re-scaffold them.

- **Frontend:** partial — scaffold React 19 + Vite (`src/App.tsx`): demonstracyjny formularz `greet` i baner aktualizacji; brak ekranów domenowych.
- **Backend / API:** partial — Tauri 2 (`src-tauri/src/lib.rs`): tylko szablonowa komenda `greet`; zarejestrowane wtyczki updater i opener.
- **Data:** absent — brak składowania danych, schematu i migracji.
- **Auth:** absent — brak hasła, wyprowadzania klucza i szyfrowania danych w spoczynku (FR-001).
- **Deploy / infra:** present — `.github/workflows/release.yml` + Tauri updater; test aktualizacji 0.1.0 → 0.1.1 zaliczony (`context/deployment/deploy-plan.md`).
- **Observability:** absent — zgodnie z PRD (zakaz wysyłki danych na zewnątrz); brak potrzeby osobnej warstwy w tym kamieniu milowym.
- **Tests:** absent — brak testów i runnera po stronie frontendu; `cargo test` dostępne, ale bez testów.

## Foundations

Brak osobnych fundamentów w tym kamieniu milowym. Jedyne brakujące warstwy (dane i hasło) mają widoczny dla użytkownika odpowiednik w FR-001, więc wchodzą pionowo w `S-01`, a testy reguły spalania wchodzą razem z regułą w `S-02`. Wdrożenie i aktualizacje są już gotowe (patrz Baseline).

## Slices

### S-01: Hasło odblokowuje zaszyfrowane dane

- **Outcome:** użytkownik może przy pierwszym uruchomieniu ustawić hasło, a przy każdym kolejnym odblokować nim aplikację; bez poprawnego hasła dane na dysku są nieczytelne, a aktualizacja aplikacji nie narusza zapisanych danych.
- **Change ID:** password-unlocked-vault
- **PRD refs:** FR-001
- **Prerequisites:** —
- **Parallel with:** —
- **Blockers:** —
- **Unknowns:**
  - Jak wyprowadzić klucz z hasła i szyfrować dane w spoczynku tak, by błędne hasło dawało jednoznaczny komunikat, a nigdy nie uszkadzało danych? — Owner: user (badanie w `/10x-plan`). Block: no.
  - Jak chronić jedyną kopię historii, gdy nowa wersja aplikacji zmienia strukturę danych (np. zaszyfrowana kopia przed zmianą)? — Owner: user. Block: no.
- **Risk:** Musi być pierwsze, bo każdy kolejny kawałek zapisuje dane, a szyfrowanie dołożone później wymagałoby konwersji jedynej kopii historii; to też miejsce głównego ryzyka (umiejętności — kryptografia w Ruście).
- **Status:** ready

### S-02: Pierwsze tankowanie daje spalanie

- **Outcome:** użytkownik może przy pierwszym uruchomieniu podać punkt startowy (data i przebieg ostatniego tankowania), dodać wpis tankowania (rodzaj paliwa, data, przebieg, litry, kwota, czy do pełna) i bez dodatkowej akcji zobaczyć spalanie tego paliwa oraz koszt.
- **Change ID:** first-fillup-consumption
- **PRD refs:** US-01, FR-002, FR-003, FR-007
- **Prerequisites:** S-01
- **Parallel with:** S-05
- **Blockers:** —
- **Unknowns:**
  - Co dokładnie znaczy „koszt” w kryterium sukcesu (kwota tankowania, koszt okresu, cena za litr)? Koszt w zł/100 km jest odłożony (Parked). — Owner: user. Block: no.
  - Jak traktować wpis z przebiegiem niższym niż poprzedni lub z datą sprzed punktu startowego — odrzucić czy ostrzec? — Owner: user. Block: no.
- **Risk:** Gwiazda przewodnia; parowanie tankowań do pełna tego samego paliwa to miejsce, gdzie najłatwiej o błędną liczbę — testy reguły powstają tu, nie później.
- **Status:** proposed

### S-03: Historia i spalanie obu paliw

- **Outcome:** użytkownik może przeglądać historię tankowań oraz bieżące spalanie osobno dla gazu i dla benzyny, także przy przeplatanych tankowaniach obu paliw i częściowych dolewkach.
- **Change ID:** fillup-history-both-fuels
- **PRD refs:** US-01, FR-007
- **Prerequisites:** S-02
- **Parallel with:** S-05, S-06, S-07, S-08
- **Blockers:** —
- **Unknowns:**
  - Jak zakomunikować brak wyniku dla paliwa, które nie ma jeszcze zamkniętego okresu (np. same dolewki), skoro nie wolno pokazać szacunku? — Owner: user. Block: no.
- **Risk:** Oddzielone od `S-02`, żeby gwiazda przewodnia była wąska; przeplatanie paliw i dolewki to scenariusze, które ujawniają błędy reguły.
- **Status:** proposed

### S-04: Poprawka i usunięcie tankowania

- **Outcome:** użytkownik może poprawić lub usunąć wpis tankowania, a spalanie obu paliw przelicza się ponownie.
- **Change ID:** correct-or-delete-fillup
- **PRD refs:** US-01, FR-004, FR-005
- **Prerequisites:** S-03
- **Parallel with:** S-05, S-06, S-07, S-08
- **Blockers:** —
- **Unknowns:**
  - Czy usunięcie wymaga potwierdzenia lub możliwości cofnięcia, skoro aplikacja jest jedynym źródłem historii? — Owner: user. Block: no.
- **Risk:** Pierwszy destrukcyjny zapis na jedynej kopii historii; przeliczenie po edycji musi dawać ten sam wynik, co wpisanie danych od nowa.
- **Status:** proposed

### S-05: Planowanie serwisu

- **Outcome:** użytkownik może dodać zaplanowany przegląd lub wymianę z progiem przebiegu, progiem daty albo obydwoma, oznaczyć go jako cykliczny, a potem edytować lub usunąć w rejestrze serwisów.
- **Change ID:** plan-service
- **PRD refs:** US-02, FR-008, FR-009
- **Prerequisites:** S-01
- **Parallel with:** S-02, S-03, S-04, S-06
- **Blockers:** —
- **Unknowns:** —
- **Risk:** Niezależny od reguły spalania, więc może iść równolegle z gwiazdą przewodnią; oznaczenie „cykliczny” samo nie tworzy kolejnego wystąpienia (US-02).
- **Status:** proposed

### S-06: Samodzielny wpis przebiegu

- **Outcome:** użytkownik może dodać samodzielny wpis przebiegu (data, przebieg) niezwiązany z tankowaniem i zobaczyć najnowszy znany przebieg auta, pochodzący z tankowania albo z takiego wpisu.
- **Change ID:** standalone-odometer-entry
- **PRD refs:** US-03, FR-006
- **Prerequisites:** S-02
- **Parallel with:** S-03, S-04, S-05
- **Blockers:** —
- **Unknowns:**
  - Co jest „najnowszym znanym przebiegiem”, gdy daty i przebiegi wpisów są niespójne (najpóźniejsza data czy najwyższy przebieg)? — Owner: user. Block: no.
- **Risk:** Wydzielone z przypomnień, bo to osobna akcja użytkownika; od tej wartości zależy niezawodność `S-07`.
- **Status:** proposed

### S-07: Przypomnienie przy otwarciu aplikacji

- **Outcome:** użytkownik po otwarciu aplikacji od razu, bez wchodzenia w osobny widok, widzi przypomnienie o każdym serwisie, którego pierwszy z progów (przebieg lub data) został osiągnięty — przy każdym otwarciu, dopóki serwis nie zostanie wykonany.
- **Change ID:** service-reminder-on-open
- **PRD refs:** US-03, FR-010
- **Prerequisites:** S-05, S-06
- **Parallel with:** S-03, S-04
- **Blockers:** —
- **Unknowns:** —
- **Risk:** Guardrail: pominięcie przypomnienia przy otwartej aplikacji to awaria; przypadki graniczne (próg osiągnięty dokładnie, oba progi naraz) wymagają testów.
- **Status:** proposed

### S-08: Wykonanie serwisu i kolejne wystąpienie

- **Outcome:** użytkownik może oznaczyć serwis jako wykonany, co gasi przypomnienie; dla serwisu cyklicznego powstaje kolejne wystąpienie z progami liczonymi od bieżącego przebiegu i bieżącej daty.
- **Change ID:** complete-recurring-service
- **PRD refs:** US-03, FR-011
- **Prerequisites:** S-07
- **Parallel with:** S-03, S-04
- **Blockers:** —
- **Unknowns:** —
- **Risk:** Łatwo pomylić podstawę progu — PRD wprost wymaga liczenia od bieżącego stanu, nie od progu nominalnego (dryf harmonogramu przyjęty świadomie).
- **Status:** proposed

## Backlog Handoff

| Roadmap ID | Change ID                  | Suggested issue title                                         | Ready for `/10x-plan` | Notes                                  |
| ---------- | -------------------------- | ------------------------------------------------------------- | --------------------- | -------------------------------------- |
| S-01       | password-unlocked-vault    | Hasło przy starcie i szyfrowanie danych w spoczynku           | yes                   | Run `/10x-plan password-unlocked-vault` |
| S-02       | first-fillup-consumption   | Punkt startowy i pierwsze tankowanie ze spalaniem i kosztem   | no                    | Czeka na S-01                          |
| S-03       | fillup-history-both-fuels  | Historia tankowań i spalanie osobno dla gazu i benzyny        | no                    | Czeka na S-02                          |
| S-04       | correct-or-delete-fillup   | Edycja i usuwanie tankowania z przeliczeniem spalania         | no                    | Czeka na S-03                          |
| S-05       | plan-service               | Rejestr serwisów: dodawanie, edycja, usuwanie, cykliczność    | no                    | Czeka na S-01                          |
| S-06       | standalone-odometer-entry  | Samodzielny wpis przebiegu i najnowszy znany przebieg         | no                    | Czeka na S-02                          |
| S-07       | service-reminder-on-open   | Przypomnienie serwisowe przy otwarciu aplikacji               | no                    | Czeka na S-05, S-06                    |
| S-08       | complete-recurring-service | Oznaczanie serwisu jako wykonany i kolejne wystąpienie        | no                    | Czeka na S-07                          |

## Open Roadmap Questions

PRD nie ma otwartych pytań (cztery rozstrzygnięte 2026-09-19). Pytania poniżej wyszły przy dekompozycji; żadne nie blokuje planowania.

1. **Czy punkt startowy i samodzielny wpis przebiegu można poprawiać lub usuwać?** PRD przewiduje edycję tylko dla tankowań (FR-004/005) i serwisów (FR-009). Domyślnie: nie, zgodnie z literą PRD. — Owner: user. Block: S-02, S-06 (nieblokujące).
2. **Czy lokalna, zaszyfrowana tym samym hasłem kopia danych przed zmianą struktury mieści się w zakazie „eksportu awaryjnego” z Access Control?** Kopia nie daje dostępu bez hasła, więc nie jest ścieżką odzyskiwania. — Owner: user. Block: roadmap-wide (nieblokujące; rozstrzygnąć w `/10x-plan password-unlocked-vault`).

## Parked

- **Wykres trendu spalania w czasie** — Why parked: PRD §Success Criteria / Secondary.
- **Koszt jazdy w zł/100 km osobno dla gazu i benzyny** — Why parked: PRD §Success Criteria / Secondary.
- **Skanowanie paragonów** — Why parked: PRD §Success Criteria / Secondary; musiałoby działać w pełni offline.
- **Wbudowana baza harmonogramów serwisowych** — Why parked: PRD §Non-Goals.
- **Odczyt z komputera pokładowego** — Why parked: PRD §Non-Goals.
- **Wiele pojazdów** — Why parked: PRD §Non-Goals.
- **Współdzielenie i wielu użytkowników** — Why parked: PRD §Non-Goals.
- **Synchronizacja w chmurze i wiele urządzeń** — Why parked: PRD §Non-Goals.
- **Przypomnienia poza aplikacją** — Why parked: PRD §Non-Goals (wymagałyby pracy w tle i wysyłki danych).
- **Aplikacja mobilna** — Why parked: PRD §Non-Goals.
- **Pełna dostępność (WCAG)** — Why parked: PRD §Non-Goals.
- **Podpisywanie kodu instalatora i winget** — Why parked: `context/deployment/deploy-plan.md` §Granice (poza zakresem; koszt certyfikatu).

## Milestone History

## Done
