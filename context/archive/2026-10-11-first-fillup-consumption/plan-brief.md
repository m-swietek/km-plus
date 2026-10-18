# Pierwsze tankowanie daje spalanie (S-02) — Plan Brief

> Full plan: `context/changes/first-fillup-consumption/plan.md`

## What & Why

Gwiazda przewodnia roadmapy: użytkownik podaje jednorazowo punkt startowy, dodaje tankowanie i od razu widzi spalanie tego paliwa oraz koszt — pierwsze główne kryterium sukcesu z PRD. To miejsce, gdzie reguła parowania tankowań do pełna najłatwiej daje błędną liczbę, a błędna liczba jest gorsza niż brak liczby, więc reguła powstaje razem z testami, w tym porównaniem z wynikami policzonymi ręcznie.

## Starting Point

Działa i jest wydany (0.1.3) zaszyfrowany sejf z S-01 z ładunkiem `format_version 1` (tylko `created_at`) i pustą powłoką „kmPlus / Brak wpisów”. Zainstalowana kopia ma prawdziwy plik sejfu v1, więc to pierwsza realna migracja formatu.

## Desired End State

Po odblokowaniu: ekran punktu startowego z potwierdzeniem, potem karty Gaz/Benzyna z najnowszym spalaniem i formularz tankowania. Zapis tankowania do pełna pokazuje l/100 km, koszt okresu i cenę za litr; dolewka — „brak wyniku” i cenę za litr. Niespójne wpisy są odrzucane z komunikatem. Aktualizacja 0.1.3 → 0.1.4 zachowuje dane i tworzy kopię pliku v1 przed pierwszym zapisem v2.

## Key Decisions Made

| Decision | Choice | Why (1 sentence) |
| --- | --- | --- |
| Znaczenie „koszt” | Koszt okresu (suma kwot paliwa w okresie) + cena za litr wpisu | Koszt liczony na tych samych wpisach co spalanie, cena za litr działa też dla dolewek. |
| Punkt startowy | Wirtualne tankowanie do pełna obu paliw | Pierwsze pełne tankowanie każdego paliwa od razu daje wynik (PRD). |
| Dolewki | Wliczane do okresu (metoda pełna-do-pełnej) | Jedyna poprawna metoda; pominięcie zaniżałoby spalanie. |
| Spójność wpisów | Odrzucenie, gdy przebieg i data nie rosną razem lub wpis jest przed startem; wpisy wsteczne pomiędzy dozwolone | Błędna liczba nie może powstać z niespójnych danych. |
| Ten sam przebieg | Różne paliwa (ta sama data) tak, to samo paliwo nie | Tankowanie obu paliw na jednej stacji jest realne; okres 0 km nie. |
| Edycja punktu startowego | Tylko potwierdzenie przed zapisem; edycja → Parked | Litera PRD teraz, usprawnienie zapisane na później. |
| Widoczność | Wynik po zapisie + najnowsze spalanie per paliwo w powłoce; bez listy (S-03) | Wynik przeżywa restart, kawałek zostaje wąski. |
| Precyzja | l/100 km, zł, zł/l — 2 miejsca; dane jako km/centylitry/grosze (liczby całkowite) | Zgodne z paragonem, bez błędów zmiennoprzecinkowych w danych. |
| Format danych | `format_version` 2 + migrator v1→v2 + kopia `kmplus.vault.pre-v2-<s>` przed pierwszym zapisem | Spełnia rejestr ryzyk; starsza aplikacja widzi `tooNew` zamiast gubić wpisy. |
| Testy | Nazwane przypadki + test wyroczni (10 wpisów z ręcznie policzonymi wynikami) | Wyrocznia sprawdza liczbę względem arkusza, nie tylko względem rozumienia reguły. |
| Wydanie | 0.1.4 + aktualizacja z 0.1.3 na prawdziwym sejfie | Migracja udowodniona tam, gdzie jest ryzyko; pierwszy zielony przebieg bramki `cargo test`. |

## Scope

**In scope:**
- Model v2 (punkt startowy, tankowania z `id`), migrator, kopia przed migracją, transakcyjny zapis
- Moduł `fuel`: walidacja i reguła pełna-do-pełnej, testy z wyrocznią
- Komendy `fuel_overview`, `set_start_point`, `add_fill_up` + rejestr kontraktów
- Ekran punktu startowego, formularz tankowania, panel wyniku, karty paliw
- Wydanie 0.1.4 i dowód migracji

**Out of scope:**
- Edycja punktu startowego (Parked), historia tankowań (S-03), edycja/usuwanie (S-04), wpis przebiegu (S-06)
- Koszt zł/100 km, wykres trendu, wymuszanie „oba baki pełne”, odrzucanie dat z przyszłości
- Runner testów frontendu, nowe zależności

## Architecture / Approach

Czysty moduł Rusta `fuel` liczy wyniki od zera z punktu startowego i całej listy tankowań (bez zapamiętanych wyników, więc S-04 dostaje przeliczanie za darmo). Sejf dekoduje plik v1 jako v2 w pamięci — odblokowanie dalej nic nie zapisuje — a pierwszy `save()` najpierw odkłada bajty pliku v1 jako `pre-v2-*`. Komendy zmieniają dane przez `UnlockedVault::commit`, który przy błędzie zapisu przywraca poprzednie dane w pamięci. Frontend zamienia tekst (przecinek lub kropka) na liczby całkowite parsowaniem tekstu i formatuje wyniki w `pl-PL`.

## Phases at a Glance

| Phase | What it delivers | Key risk |
| --- | --- | --- |
| 1. Model v2, migracja, reguła | `VaultData` v2, migrator, kopia `pre-v2`, moduł `fuel`, testy + wyrocznia | Pomyłka w regule wspólna dla kodu i testów — łapie ją wyrocznia potwierdzona przez użytkownika |
| 2. Komendy Tauri | 3 komendy, testy kształtu JSON, rejestr kontraktów | Rozjazd nazw pól z frontendem (brak kontroli przy kompilacji) |
| 3. UI | Punkt startowy z potwierdzeniem, formularz, wynik, karty | Parsowanie liczb bez testów automatycznych — pokryte scenariuszami ręcznymi |
| 4. Wydanie 0.1.4 | Zielony CI z testami, aktualizacja 0.1.3 → 0.1.4, dowód SHA-256 | Migracja jedynej kopii historii — kopia `pre-v2` i sprawdzenie na prawdziwym pliku |

**Prerequisites:** S-01 wydany (0.1.3 zainstalowana z sejfem); cargo w PATH (`$HOME\.cargo\bin`); `gh` zalogowany; zgoda użytkownika na start workflow i publikacja draftu ręką użytkownika.
**Estimated effort:** ~3–4 sesje w 4 fazach (Faza 4 to głównie czekanie na CI i testy ręczne).

## Open Risks & Assumptions

- Próbka wyroczni jest ułożona przez planistę; jej wartość zależy od potwierdzenia przez użytkownika względem arkusza (krok 1.4).
- Punkt startowy zakłada pełne oba baki; jeśli użytkownik poda chwilę, gdy tylko jeden był pełny, pierwszy wynik drugiego paliwa będzie zafałszowany (tylko tekst ostrzegawczy).
- Literówka w punkcie startowym zostaje do czasu usprawnienia z Parked.
- Po zapisie v2 powrót do 0.1.3 jest możliwy tylko ręcznie z pliku `pre-v2-*`.

## Success Criteria (Summary)

- Użytkownik podaje punkt startowy, dodaje pierwsze tankowanie do pełna i od razu widzi poprawne spalanie, koszt okresu i cenę za litr.
- Wyniki zgadzają się z ręcznie policzoną próbką (i arkuszem użytkownika), a niespójne wpisy nie trafiają do danych.
- Prawdziwy sejf z 0.1.3 przechodzi aktualizację do 0.1.4 bez utraty danych, z kopią pliku v1.
