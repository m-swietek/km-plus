---
project: "kmPlus"
version: 1
status: draft
created: 2026-09-19
context_type: greenfield
product_type: desktop
target_scale:
  users: small
timeline_budget:
  mvp_weeks: 12
  hard_deadline: 2026-12-06
  after_hours_only: true
---

# PRD: kmPlus

## Vision & Problem Statement

Właściciel samochodu, który sam nim jeździ, dziś nie ma jednego miejsca, w którym zbierałyby się dane z paragonów za paliwo i informacje o przebiegu / terminach przeglądów i wymian części. Brak śledzenia zużycia paliwa utrudnia łatwe rozliczanie kosztów podróży, gdy podwozi inne osoby, a brak przypomnień o zbliżającym się przeglądzie lub wymianie części prowadzi do sytuacji, gdy termin zostaje przegapiony — co frustruje i bywa kosztowne.

Ten projekt powstaje przede wszystkim jako projekt do nauki (budowa własnej aplikacji desktopowej od zera, z pomocą agenta AI) — autor nie porównywał gotowych aplikacji do śledzenia paliwa pod kątem tego, czego im brakuje; wartością jest połączenie w jednym, własnym narzędziu dwóch rzeczy, które dziś rozwiązuje osobno lub wcale: rejestru tankowań/spalania oraz przypomnień serwisowych opartych na przebiegu lub czasie.

## User & Persona

Jedna osoba: sam autor, jako kierowca i właściciel samochodu. Sięga po aplikację w dwóch momentach: (1) po zatankowaniu, żeby wpisać dane z paragonu i mieć bieżący obraz spalania oraz kosztów — także pod kątem rozliczenia z pasażerami; (2) przy wprowadzaniu aktualnego przebiegu, żeby sprawdzić lub zaktualizować status nadchodzących przeglądów/wymian części.

## Success Criteria

### Primary
- Pierwszy przepływ działa: logowanie hasłem → dodanie wpisu tankowania (przebieg, litry) → wykres i dane o średnim spalaniu aktualizują się automatycznie.
- Pełne MVP działa: powyższe + rejestr przeglądów/wymian części z regułą przypominania na bazie przebiegu lub czasu.

### Secondary
- Wykresy/statystyki trendu spalania w czasie (np. miesiąc do miesiąca).

### Guardrails
- Dane tankowań i przeglądów nigdy nie giną — trwały zapis, utrata historii jest krytyczną awarią.
- Przypomnienia o zbliżającym się przeglądzie/wymianie nie mogą zawodzić — nie wolno przegapić terminu z winy aplikacji.

## User Stories

### US-01: Użytkownik dodaje tankowanie i widzi zaktualizowane spalanie

- **Given** zalogowany użytkownik z co najmniej jednym wcześniejszym wpisem przebiegu
- **When** dodaje nowy wpis tankowania (data, przebieg, litry, kwota)
- **Then** wpis zapisuje się trwale, a historia tankowań i wykres/dane o średnim spalaniu aktualizują się automatycznie

#### Acceptance Criteria
- Błędny wpis (np. przebieg mniejszy niż poprzedni) można później edytować bez utraty pozostałej historii
- Wykres/dane o spalaniu przeliczają się natychmiast po zapisaniu wpisu

# TODO: user story covering the przeglądy/przypomnienia flow (FR-006, FR-007) — see Open Questions

## Functional Requirements

### Dostęp

- FR-001: Użytkownik może zalogować się hasłem. Priority: must-have
  > Socrates: Kontrargument rozważony: "hasło to zbędne tarcie / pozorne bezpieczeństwo dla appki jednoosobowej." Rozwiązanie: zostaje — użytkownik chce ekranu logowania; niezalogowany użytkownik nie powinien mieć dostępu do danych.

### Tankowania

- FR-002: Użytkownik może dodać wpis tankowania (data, przebieg, litry, kwota). Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-003: Użytkownik może edytować istniejący wpis tankowania (np. po pomyłce). Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-004: Użytkownik może przeglądać historię tankowań w formie tabeli oraz aktualne średnie spalanie. Priority: must-have
  > Socrates: Kontrargument rozważony: "wykres na starcie to nadmiarowa złożoność — wystarczy tabela liczb." Rozwiązanie: tabela + wyliczone średnie spalanie wchodzą do MVP jako must-have; wykres trendu przesunięty do Secondary (patrz Success Criteria).

### Przebieg

- FR-005: Użytkownik może dodać samodzielny wpis przebiegu (data, przebieg) niezależny od tankowania, tak aby przypomnienia serwisowe nie zależały wyłącznie od tankowań. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.

### Przeglądy i przypomnienia

- FR-006: Użytkownik może dodać lub edytować zaplanowany przegląd/wymianę części z regułą przypominania opartą na przebiegu lub czasie. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-007: Aplikacja wyświetla przypomnienie w interfejsie przy otwarciu aplikacji, gdy zbliża się lub mija termin przeglądu/wymiany, na podstawie najnowszego znanego przebiegu (z tankowania lub samodzielnego wpisu) lub daty. Priority: must-have
  > Socrates: Kontrargument rozważony: "przypomnienie tylko w UI aplikacji można łatwo przeoczyć, jeśli aplikacja jest rzadko otwierana." Rozwiązanie: zostaje jako przypomnienie w UI przy otwarciu — ryzyko przeoczenia zaakceptowane dla MVP, bo aplikacja i tak jest otwierana przy każdym tankowaniu. Kanały zewnętrzne (e-mail/powiadomienie systemowe) świadomie poza zakresem MVP.

## Non-Functional Requirements

- Dane o tankowaniach, przebiegu i przeglądach nie opuszczają urządzenia użytkownika — brak wysyłki do zewnętrznych usług.
- Wszystkie funkcje MVP (dodawanie wpisów, przeglądanie historii, przypomnienia) działają bez połączenia z internetem.

## Business Logic

Aplikacja porównuje aktualny przebieg/datę z zapisanymi progami przeglądów i wymian części, i sama decyduje, kiedy pokazać przypomnienie.

Reguła konsumuje: najnowszy znany przebieg (z wpisu tankowania lub samodzielnego wpisu przebiegu), bieżącą datę oraz zdefiniowany dla danego przeglądu/części próg (przebieg docelowy i/lub data docelowa). Wyjściem reguły jest widoczne w interfejsie przypomnienie, gdy próg zostanie osiągnięty lub przekroczony przez którykolwiek z dwóch warunków (przebieg lub czas) — nie wymaga to obu naraz. Użytkownik spotyka tę regułę przy każdym otwarciu aplikacji, jako element widoczny bez dodatkowej akcji z jego strony.

## Access Control

Logowanie hasłem, jeden płaski model konta — bez ról ani poziomów dostępu. Hasło chroni dane głównie przed przypadkowym dostępem innych domowników korzystających z tego samego komputera, nie przed wieloma użytkownikami aplikacji. Brak rejestracji/self-service — jedno predefiniowane konto właściciela.

## Non-Goals

- Brak skanowania/OCR paragonów — dane z paragonów wpisywane są zawsze ręcznie.
- Brak synchronizacji w chmurze i obsługi wielu urządzeń — dane trzymane lokalnie, jedno urządzenie.
- Brak wielu użytkowników i ról — jedno konto właściciela, bez udostępniania danych innym osobom.
- Brak powiadomień zewnętrznych (e-mail/SMS/systemowych) — przypomnienia wyłącznie w interfejsie aplikacji przy jej otwarciu (patrz FR-007).

## Open Questions

1. **User Story dla przeglądów/przypomnień (FR-006, FR-007) nie została rozpisana w formacie Given/When/Then** — tylko przepływ tankowania (US-01) doczekał się pełnej historii. Owner: user. Block: no (FR-006/FR-007 są już opisane w Functional Requirements, ale brak dedykowanej user story utrudni downstream planowanie akceptacji).
