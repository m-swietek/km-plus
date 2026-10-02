---
project: "kmPlus"
version: 2
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

### US-02: Użytkownik planuje przegląd lub wymianę części

- **Given** zalogowany użytkownik
- **When** dodaje zaplanowany przegląd/wymianę części, podając próg przebiegu i/lub próg czasu (data), oraz opcjonalnie zaznacza, że serwis jest cykliczny (np. co 10 000 km lub co N miesięcy)
- **Then** wpis pojawia się w zakładce serwisów, a logika przypomnień zaczyna śledzić oba zdefiniowane progi (lub jeden, jeśli podano tylko jeden)

#### Acceptance Criteria
- Można podać tylko próg przebiegu, tylko próg czasu, lub oba naraz
- Gdy podano oba progi, przypomnienie wyzwala pierwsze osiągnięte kryterium; drugie przestaje być śledzone do czasu oznaczenia wpisu jako wykonany (patrz FR-008)
- Zaznaczenie "cykliczny" nie tworzy jeszcze kolejnego wystąpienia — to następuje dopiero przy oznaczeniu bieżącego wpisu jako wykonany (patrz FR-008)
- Wpis można edytować po zapisaniu (np. korekta progu)

### US-03: Użytkownik widzi przypomnienie o zbliżającym się lub minionym przeglądzie

- **Given** zalogowany użytkownik z co najmniej jednym zaplanowanym przeglądem/wymianą, którego próg (przebieg lub czas) został osiągnięty lub przekroczony
- **When** otwiera aplikację
- **Then** widzi przypomnienie w interfejsie bez potrzeby dodatkowej akcji, oparte na najnowszym znanym przebiegu (z tankowania lub samodzielnego wpisu) lub bieżącej dacie

#### Acceptance Criteria
- Przypomnienie pojawia się automatycznie przy każdym otwarciu aplikacji, dopóki wpis nie zostanie oznaczony jako wykonany
- Jeśli żaden próg nie został osiągnięty, przypomnienie się nie pojawia

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

- FR-006: Użytkownik może dodać lub edytować zaplanowany przegląd/wymianę części z regułą przypominania opartą na przebiegu i/lub czasie, w tym oznaczyć wpis jako cykliczny (np. co 10 000 km lub co N miesięcy). Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-007: Aplikacja wyświetla przypomnienie w interfejsie przy otwarciu aplikacji, gdy zbliża się lub mija termin przeglądu/wymiany, na podstawie najnowszego znanego przebiegu (z tankowania lub samodzielnego wpisu) lub daty. Priority: must-have
  > Socrates: Kontrargument rozważony: "przypomnienie tylko w UI aplikacji można łatwo przeoczyć, jeśli aplikacja jest rzadko otwierana." Rozwiązanie: zostaje jako przypomnienie w UI przy otwarciu — ryzyko przeoczenia zaakceptowane dla MVP, bo aplikacja i tak jest otwierana przy każdym tankowaniu. Kanały zewnętrzne (e-mail/powiadomienie systemowe) świadomie poza zakresem MVP.
- FR-008: Użytkownik może oznaczyć zaplanowany przegląd/wymianę jako wykonany; dla wpisów oznaczonych jako cykliczne aplikacja automatycznie tworzy kolejne wystąpienie z nowym progiem wyliczonym od bieżącego przebiegu/daty. Priority: must-have
  > Socrates: Kontrargument rozważony: "automatyczne tworzenie kolejnego wpisu to ukryta magia, którą użytkownik może przeoczyć." Rozwiązanie: zostaje jako must-have — bez tej funkcji cykliczność zdefiniowana w FR-006 byłaby jedynie etykietą bez realnej wartości; nowy wpis pojawia się widocznie w zakładce serwisów tak samo jak każdy inny.

## Non-Functional Requirements

- Dane o tankowaniach, przebiegu i przeglądach nie opuszczają urządzenia użytkownika — brak wysyłki do zewnętrznych usług.
- Wszystkie funkcje MVP (dodawanie wpisów, przeglądanie historii, przypomnienia) działają bez połączenia z internetem.

## Business Logic

Aplikacja porównuje aktualny przebieg/datę z zapisanymi progami przeglądów i wymian części, i sama decyduje, kiedy pokazać przypomnienie.

Reguła konsumuje: najnowszy znany przebieg (z wpisu tankowania lub samodzielnego wpisu przebiegu), bieżącą datę oraz zdefiniowany dla danego przeglądu/części próg (przebieg docelowy i/lub data docelowa). Wyjściem reguły jest widoczne w interfejsie przypomnienie, gdy próg zostanie osiągnięty lub przekroczony przez którykolwiek z dwóch warunków (przebieg lub czas) — nie wymaga to obu naraz. Użytkownik spotyka tę regułę przy każdym otwarciu aplikacji, jako element widoczny bez dodatkowej akcji z jego strony.

Gdy dla jednego przeglądu/wymiany zdefiniowano oba progi (przebieg i czas), pierwsze osiągnięte kryterium wyzwala przypomnienie, a drugie przestaje być śledzone do momentu oznaczenia wpisu jako wykonany. Dla wpisów oznaczonych jako cykliczne, oznaczenie jako wykonany tworzy automatycznie kolejne wystąpienie z nowym progiem wyliczonym od bieżącego przebiegu/daty — użytkownik nie musi ręcznie zakładać kolejnego wpisu.

## Access Control

Logowanie hasłem, jeden płaski model konta — bez ról ani poziomów dostępu. Hasło chroni dane głównie przed przypadkowym dostępem innych domowników korzystających z tego samego komputera, nie przed wieloma użytkownikami aplikacji. Brak rejestracji/self-service — jedno predefiniowane konto właściciela.

## Non-Goals

- Brak skanowania/OCR paragonów — dane z paragonów wpisywane są zawsze ręcznie.
- Brak synchronizacji w chmurze i obsługi wielu urządzeń — dane trzymane lokalnie, jedno urządzenie.
- Brak wielu użytkowników i ról — jedno konto właściciela, bez udostępniania danych innym osobom.
- Brak powiadomień zewnętrznych (e-mail/SMS/systemowych) — przypomnienia wyłącznie w interfejsie aplikacji przy jej otwarciu (patrz FR-007).

## Open Questions

Brak otwartych pytań — wejście z shape-notes.md było w pełni ukształtowane (4/4 heurystyki), a poprzednia luka (brak dedykowanej user story dla przeglądów/przypomnień) została zamknięta przez US-02 i US-03.
