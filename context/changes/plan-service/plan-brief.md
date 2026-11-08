# Rejestr serwisów — Plan Brief

> Full plan: `context/changes/plan-service/plan.md`

## What & Why

S-05 (US-02, FR-008, FR-009): użytkownik planuje przegląd lub wymianę z progiem przebiegu, terminem albo obydwoma, może oznaczyć wpis jako cykliczny, a potem go edytować lub usunąć. To pierwsza połowa drugiego głównego kryterium sukcesu z PRD — bez zapisanych progów S-07 nie ma czego przypominać, a S-08 nie ma od czego liczyć kolejnego wystąpienia.

## Starting Point

Wersja 0.1.6: sejf v2 z punktem startowym i tankowaniami (`vault/data.rs:38-54`), działający mechanizm migracji z kopią `pre-v<N>` przy pierwszym zapisie, jedna strona paliwowa w `AppShell`. Serwisów nie ma ani w danych, ani w UI.

## Desired End State

Po odblokowaniu są zakładki „Paliwo” (bez zmian) i „Serwisy”. W „Serwisach” formularz dodaje i edytuje wpisy (nazwa, notatka, próg km i/lub termin, cykliczność z interwałem), lista pokazuje je w stałej kolejności, a usunięcie wymaga potwierdzenia w wierszu. Dane przetrwały aktualizację: sejf jest v3, a kopia `pre-v3-*` jest identyczna z plikiem sprzed aktualizacji.

## Key Decisions Made

| Decision | Choice | Why (1 sentence) |
| --- | --- | --- |
| Cykliczność | Interwał dla każdego zadanego progu: „co N km” przy progu przebiegu, „co M miesięcy” przy terminie | S-08 policzy kolejne wystąpienie od bieżącego stanu bez pytania użytkownika i bez kolejnej migracji. |
| Pola wpisu | Nazwa (wymagana, 1–100 znaków po przycięciu) + notatka (opcjonalna) | Obejmuje przeglądy i wymiany bez taksonomii, która nie steruje żadnym zachowaniem. |
| Progi już osiągnięte | Przyjmowane tak, jak wpisane — bez porównania z przebiegiem i dzisiejszą datą | Zaległy serwis da się zapisać, a S-05 nie zależy od definicji „najnowszego przebiegu” (S-06). |
| Usuwanie | Dwuetapowe w wierszu: „Usuń” → „Usunąć serwis „<nazwa>”? [Usuń] [Anuluj]”; `notFound` → „Ten wpis już nie istnieje.” | Chroni jedyną kopię danych przed pomyłką bez wtyczki dialogów; ten sam wzorzec i komunikat co w S-04. |
| Kolejność listy | Wpisy z terminem rosnąco po dacie, potem same progi km rosnąco; remis po `id` | Deterministyczna i zrozumiała bez znajomości bieżącego przebiegu. |
| Układ | Zakładki „Paliwo” / „Serwisy”, domyślnie „Paliwo”, stan lokalny | Odpowiada dwóm momentom użycia z PRD; rejestr nie ginie pod rosnącymi historiami. |
| Wydanie | Własne: 0.1.7, jeśli S-05 scali się pierwszy, inaczej kolejny numer po S-04 | Migracja v2 → v3 udowodniona osobno na prawdziwych danych. |
| Format danych | v3 z `services` + `next_service_id`, migrator v2 → v3 (łańcuch od v1) | Reguła z S-02: nowe pole = nowa wersja + migrator, nie `serde(default)`. |
| Komendy | Osobne `service_register` / `add_service` / `update_service` / `delete_service`, IO w `spawn_blocking` | Nie dotyka `fuel_overview` (S-04) i nie powtarza F4 (blokujący zapis w `async`). |
| Wartości graniczne (agent) | Interwał km 1–9 999 999, miesiące 1–120, notatka ≤ 500 znaków, pusta → brak | Zakresy jak przy przebiegu; niezakwestionowane przez użytkownika. |

## Scope

**In scope:** domena i walidacja serwisu; sejf v3 z migracją i testami kopii; cztery komendy z kontraktem; zakładki, formularz dodawania/edycji, lista z usuwaniem; wydanie z dowodem migracji.

**Out of scope:** kolejne wystąpienie i arytmetyka miesięcy (S-08); przypomnienia i status „wymaga uwagi” (S-07); oznaczanie jako wykonany; ostrzeżenia o progu w przeszłości; rodzaj wpisu; cofanie usunięcia; rejestr przed punktem startowym; zmiany w paliwach.

## Architecture / Approach

Nowy moduł `src-tauri/src/service/` (lustro `fuel/`): czysta walidacja i kolejność → `VaultData` v3 → cienkie komendy `apply_*` przez `UnlockedVault::commit` w `spawn_blocking` → `src/service/` (api, zakładka, formularz, lista). Backend ustala kolejność i reguły; frontend tylko wyświetla i tłumaczy `kind` na polskie komunikaty.

## Phases at a Glance

| Phase | What it delivers | Key risk |
| --- | --- | --- |
| 1. Domena serwisu i format sejfu v3 | Typy, walidacja, kolejność; v3 + migrator v2 → v3; testy kopii `pre-v3` | Migracja gubi lub zmienia dane paliwowe |
| 2. Komendy i kontrakt serwisów | 4 komendy, DTO, `generate_handler!`, `contract-surfaces.md`, `api.ts` | Kolizja scalenia z S-04 w `lib.rs` i rejestrze |
| 3. UI rejestru serwisów i zakładki | Zakładki, formularz z warunkowymi interwałami, lista z potwierdzeniem usunięcia | Przełączanie zakładek kasuje stan formularzy |
| 4. Wydanie i dowód migracji v2 → v3 | Wydanie `app-v<X>`, dowód SHA-256 i kopii `pre-v3` | Testowy sejf zamiast prawdziwego w katalogu danych |

**Prerequisites:** S-01 (done); ustawiony punkt startowy (bramka UI); `cargo` z `$HOME\.cargo\bin`; uprawnienia `gh` do `release.yml`.
**Estimated effort:** ~3–4 sesje w 4 fazach (Faza 4 zależy od ręcznej publikacji przez użytkownika).

## Open Risks & Assumptions

- Kolizje scalenia z S-04: `lib.rs` (`generate_handler!`), tabela komend w `contract-surfaces.md`, `AppShell.tsx`, numer wersji (obie zmiany celują w 0.1.7). Blok paliwowy przenoszony bez zmian, komendy dopisane na końcu listy; style S-05 w osobnym `src/service/service.css`, więc `App.css` nie koliduje.
- Ręczna weryfikacja Fazy 3 i Faza 4 nie mogą nakładać się w czasie z tymi samymi fazami S-04 (wspólny katalog danych i port 1420; build deweloperski S-05 zapisuje v3). Wydania po kolei, zalecane S-04 pierwsze, potem rebase S-05 na `main`.
- Założenie: S-04 nie zmienia formatu sejfu (potwierdzone przez równoległe planowanie); inaczej jedna ze zmian staje się v4.
- Arytmetyka miesięcy odłożona do S-08; S-07 może wyróżniać wpisy bez zmiany bazowej kolejności.
- Po pierwszym zapisie v3 starsze wersje odmówią otwarcia (`tooNew`); powrót tylko przez ręczne przywrócenie `pre-v3-*`.

## Success Criteria (Summary)

- Użytkownik dodaje, edytuje i usuwa wpisy z progiem km, terminem lub obydwoma, z cyklicznością, i widzi je w ustalonej kolejności po restarcie.
- Aktualizacja na prawdziwych danych: odblokowanie nic nie pisze, pierwszy zapis tworzy `pre-v3-*` identyczny z plikiem sprzed aktualizacji, tankowania bez zmian.
- Oznaczenie „cykliczny” niczego nie tworzy i nie ma przypomnień — zgodnie z US-02.
