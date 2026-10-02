---
project: "kmPlus"
version: 4
status: draft
created: 2026-09-19
context_type: greenfield
product_type: desktop
target_scale:
  users: small
timeline_budget:
  mvp_weeks: 6
  hard_deadline: 2026-12-06
  after_hours_only: true
---

# kmPlus — Product Requirements Document

## Vision & Problem Statement

Kierowca samochodu z instalacją gazową nie zna faktycznego spalania swojego auta — komputer pokładowy podaje przekłamany wynik, bo nie rozlicza poprawnie jazdy na dwóch paliwach. Żeby mieć świadomość realnego zużycia, właściciel utrzymuje arkusz kalkulacyjny i po każdym tankowaniu przepisuje do niego dane z paragonu. Równolegle terminy przeglądów i wymian części żyją w kalendarzu albo w aplikacji typu todo-lista, czyli poza kontekstem samochodu. Kosztem jest ręczna praca przy każdym tankowaniu oraz brak jednego miejsca, w którym widać stan auta — prawda o samochodzie leży rozbita na trzy źródła, z których żadne nie jest ani kompletne, ani wiarygodne.

Autor nie porównywał gotowych aplikacji do śledzenia tankowań pod kątem tego, czego im brakuje — projekt powstaje przede wszystkim jako projekt do nauki, czyli budowa własnej aplikacji desktopowej od zera. Nie zakłada się więc przewagi nad istniejącymi narzędziami rynkowymi. Punktem odniesienia, który aplikacja musi przebić, jest dotychczasowy arkusz kalkulacyjny autora: ma dawać ten sam obraz spalania przy mniejszym nakładzie pracy i dokładać to, czego arkusz nie robi — aktywny nadzór nad nadchodzącymi przeglądami i wymianami.

## User & Persona

Jedna osoba: autor, kierowca i właściciel jednego samochodu z instalacją gazową. Sięga po aplikację w dwóch momentach:

1. **Po tankowaniu** — żeby wpisać dane z paragonu i zobaczyć, czy spalanie się zmieniło. Dziś ten moment kończy się w arkuszu kalkulacyjnym.
2. **Gdy myśli o stanie auta** — żeby sprawdzić, co go czeka: jaki przegląd lub wymiana jest przed nim i jak blisko. Dziś ten moment kończy się w kalendarzu albo nigdzie.

Bez drugiego użytkownika, bez współdzielenia danych, bez obsługi wielu pojazdów.

## Success Criteria

### Primary
- Ścieżka paliwowa działa end-to-end: logowanie hasłem → jednorazowy punkt startowy przy pierwszym uruchomieniu → dodanie wpisu tankowania → widoczne spalanie liczone osobno dla gazu i dla benzyny oraz koszt.
- Rejestr serwisowy działa: zaplanowany przegląd lub wymiana z zadanym progiem powoduje pojawienie się przypomnienia, gdy próg zostanie osiągnięty.

### Secondary
- Wykres trendu spalania w czasie (np. miesiąc do miesiąca).
- Koszt jazdy w zł/100 km osobno dla gazu i benzyny — odpowiedź na pytanie, czy instalacja gazowa się opłaca.
- Skanowanie paragonów jako metoda wpisu danych, zamiast ręcznego przepisywania.

### Guardrails
- Dane nigdy nie giną. Po porzuceniu dotychczasowego arkusza aplikacja jest jedynym źródłem historii tankowań i serwisów — utrata tej historii jest awarią krytyczną, bo nie da się jej odtworzyć.
- Przypomnienie pojawia się niezawodnie przy każdym otwarciu aplikacji, dopóki serwis nie zostanie oznaczony jako wykonany. Pominięcie przypomnienia przy otwartej aplikacji jest awarią; nieotwarcie aplikacji przez użytkownika nie jest — to ograniczenie przyjęte świadomie.
- Policzone spalanie musi być poprawne. Błędna liczba jest gorsza niż brak liczby — celem projektu jest zastąpienie przekłamującego komputera pokładowego czymś, czemu można ufać.

## User Stories

### US-01: Użytkownik dodaje tankowanie i widzi zaktualizowane spalanie

- **Given** zalogowany użytkownik, który przy pierwszym uruchomieniu podał punkt startowy (datę i przebieg ostatniego tankowania)
- **When** dodaje wpis tankowania z rodzajem paliwa, datą, przebiegiem, liczbą litrów i kwotą
- **Then** wpis zapisuje się trwale, a spalanie dla tego rodzaju paliwa oraz koszt aktualizują się i są widoczne bez dodatkowej akcji

#### Acceptance Criteria
- Spalanie dla danego paliwa liczone jest względem poprzedniego wpisu TEGO SAMEGO paliwa, nie względem wpisu sąsiedniego w czasie
- Okres rozliczeniowy spalania zamykają wyłącznie tankowania do pełna; wpis częściowej dolewki nie generuje wyniku spalania
- Pierwszy wpis tankowania po podaniu punktu startowego już daje wynik spalania
- Błędny wpis można poprawić lub usunąć, a spalanie przelicza się ponownie

### US-02: Użytkownik planuje przegląd lub wymianę części

- **Given** zalogowany użytkownik
- **When** dodaje zaplanowany przegląd lub wymianę, podając próg przebiegu, próg daty albo oba, i opcjonalnie oznacza wpis jako cykliczny
- **Then** wpis pojawia się w rejestrze serwisów, a aplikacja zaczyna śledzić zadane progi

#### Acceptance Criteria
- Można podać sam próg przebiegu, samą datę, albo oba naraz
- Wpis można później edytować (korekta progu) oraz usunąć
- Oznaczenie "cykliczny" samo w sobie nie tworzy jeszcze kolejnego wystąpienia

### US-03: Użytkownik widzi przypomnienie i zamyka serwis

- **Given** zalogowany użytkownik z zaplanowanym serwisem, którego pierwszy z zadanych progów został osiągnięty
- **When** otwiera aplikację
- **Then** widzi przypomnienie w interfejsie bez dodatkowej akcji, a po wykonaniu serwisu może oznaczyć go jako wykonany

#### Acceptance Criteria
- Przypomnienie opiera się na najnowszym znanym przebiegu — z wpisu tankowania albo z samodzielnego wpisu przebiegu — lub na bieżącej dacie
- Przypomnienie pojawia się przy każdym otwarciu aplikacji, dopóki wpis nie zostanie oznaczony jako wykonany
- Oznaczenie jako wykonany gasi przypomnienie; dla wpisu cyklicznego powstaje kolejne wystąpienie z progiem liczonym od bieżącego przebiegu i daty

## Functional Requirements

### Dostęp

- FR-001: Użytkownik może zalogować się hasłem przy starcie aplikacji; bez poprawnego hasła dane pozostają niedostępne w postaci jawnej. Priority: must-have
  > Socrates: Kontrargument rozważony: "hasło nie chroni tego, co się wydaje — kto ma konto systemowe, ten i tak odczyta plik z dysku". Rozwiązanie: FR wzmocniony — dane mają być przechowywane w postaci zaszyfrowanej, a hasło jest warunkiem ich odszyfrowania, nie tylko zasłoną na ekranie. Konsekwencja przyjęta świadomie: brak odzyskiwania hasła oznacza bezpowrotną utratę historii (rozstrzygnięte 2026-09-19, patrz Access Control).
- FR-002: Użytkownik może przy pierwszym uruchomieniu podać punkt startowy — datę i przebieg ostatniego tankowania. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.

### Tankowania i przebieg

- FR-003: Użytkownik może dodać wpis tankowania zawierający rodzaj paliwa (gaz lub benzyna), datę, przebieg, liczbę litrów, kwotę oraz informację, czy tankowano do pełna. Priority: must-have
  > Socrates: Kontrargument rozważony: "bez informacji o tankowaniu do pełna spalanie liczone z częściowej dolewki jest błędne, a aplikacja nie ma jak tego wykryć — łamie to guardrail o poprawności spalania". Rozwiązanie: FR rozszerzony o znacznik tankowania do pełna; wpisy częściowe nie zamykają okresu rozliczeniowego spalania.
- FR-004: Użytkownik może edytować istniejący wpis tankowania. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-005: Użytkownik może usunąć wpis tankowania. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-006: Użytkownik może dodać samodzielny wpis przebiegu (data, przebieg), niezwiązany z tankowaniem. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-007: Użytkownik może przeglądać historię tankowań oraz bieżące spalanie liczone osobno dla gazu i dla benzyny. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.

### Serwisy i przypomnienia

- FR-008: Użytkownik może dodać zaplanowany przegląd lub wymianę z progiem przebiegu, progiem daty albo obydwoma, oraz oznaczyć go jako cykliczny. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-009: Użytkownik może edytować i usunąć zaplanowany przegląd lub wymianę. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-010: Aplikacja pokazuje przypomnienie w interfejsie przy otwarciu aplikacji, gdy pierwszy z zadanych progów zostanie osiągnięty, na podstawie najnowszego znanego przebiegu lub bieżącej daty. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.
- FR-011: Użytkownik może oznaczyć przegląd lub wymianę jako wykonany; dla wpisów cyklicznych aplikacja automatycznie tworzy kolejne wystąpienie z progiem liczonym od bieżącego przebiegu i daty. Priority: must-have
  > Socrates: Brak kontrargumentu; zostaje jak jest.

## Non-Functional Requirements

- Dane przechowywane przez aplikację są nieczytelne bez podania hasła użytkownika — zobowiązanie binarne, niezależne od tego, kto uzyska dostęp do pliku na dysku.
- Wszystkie funkcje MVP — dodawanie wpisów, przeglądanie historii, wyliczanie spalania, przypomnienia — działają przy całkowicie odłączonej sieci.
- Dane o przebiegu, kosztach i terminach serwisowych nie są wysyłane do żadnej zewnętrznej usługi. Zobowiązanie binarne, bez wyjątków i bez furtki na zgodę użytkownika.

## Business Logic

Aplikacja wylicza faktyczne zużycie paliwa osobno dla każdego paliwa i sama decyduje, kiedy ostrzec o zbliżającym się serwisie.

Pierwsza część reguły konsumuje wpisy tankowań — rodzaj paliwa, datę, stan licznika, liczbę litrów, kwotę oraz informację, czy tankowano do pełna — a także punkt startowy podany przy pierwszym uruchomieniu. Reguła zestawia ze sobą kolejne tankowania do pełna tego samego paliwa i z dystansu między nimi oraz zatankowanej ilości wyprowadza zużycie. Wyjściem są dwie osobne wartości: zużycie gazu i zużycie benzyny. Przyjęte świadomie uproszczenie: kilometry przejechane na benzynie przy rozruchu wliczają się do dystansu przypisanego gazowi, więc zużycie gazu jest nieznacznie zawyżone. Błąd jest systematyczny, więc porównywanie okresów między sobą pozostaje wiarygodne — i to porównanie, a nie wartość bezwzględna, jest tym, czego użytkownik szuka.

Druga część reguły konsumuje najnowszy znany stan licznika — pochodzący z tankowania albo z samodzielnego wpisu przebiegu — bieżącą datę oraz progi zdefiniowane dla zaplanowanego serwisu, którymi mogą być przebieg, data albo oba naraz. Reguła wyzwala przypomnienie, gdy pierwsze z zadanych kryteriów zostanie osiągnięte lub przekroczone; nie wymaga to spełnienia obu. Wpis pozostaje w stanie wymagającym uwagi aż do oznaczenia go jako wykonany. Dla serwisów oznaczonych jako cykliczne oznaczenie wykonania tworzy kolejne wystąpienie z progiem wyliczonym od bieżącego stanu licznika i bieżącej daty — a nie od progu pierwotnie zaplanowanego. Reguła odzwierciedla realny odstęp między wykonanymi serwisami; świadomie przyjętą konsekwencją jest to, że spóźniony serwis trwale przesuwa dalszy harmonogram względem planu nominalnego.

Użytkownik spotyka pierwszą część reguły zaraz po zapisaniu wpisu tankowania — wynik pojawia się bez dodatkowej akcji z jego strony. Drugą spotyka przy otwarciu aplikacji, gdzie aktualne przypomnienia są widoczne od razu, bez wchodzenia w osobny widok.

## Access Control

Hasło podawane przy starcie aplikacji. Jedno predefiniowane konto właściciela — bez rejestracji i bez self-service. Model płaski: brak ról i poziomów dostępu, bo użytkownik jest dokładnie jeden.

Hasło nie jest samą zasłoną na ekranie: dane przechowywane są w postaci zaszyfrowanej, a ich postać jawna powstaje dopiero po podaniu poprawnego hasła.

Nie istnieje żadna ścieżka odzyskania dostępu bez hasła — ani pytanie pomocnicze, ani klucz zapasowy, ani eksport awaryjny. Jest to rozstrzygnięta właściwość produktu, nie luka: utrata hasła oznacza bezpowrotną utratę całej historii. Ryzyko zostało świadomie przeniesione poza aplikację, na zewnętrzny menedżer haseł użytkownika.

Hasło chroni dane przed przypadkowym wglądem osoby korzystającej z tego samego komputera. Nie jest mechanizmem rozdzielania wielu użytkowników aplikacji ani ochroną przed atakiem — taki poziom nie jest tu celem.

## Non-Goals

- **Bez wbudowanej bazy harmonogramów serwisowych** — aplikacja nie wie, co i co ile kilometrów wymienia się w danym modelu auta; progi wpisuje użytkownik.
- **Bez odczytu z komputera pokładowego** — stan licznika zawsze wprowadzany ręcznie, bez czytnika i bez integracji z autem.
- **Bez obsługi wielu pojazdów** — jedno auto na stałe, żadnego wyboru pojazdu w interfejsie.
- **Bez współdzielenia i wielu użytkowników** — jedno konto właściciela, bez zapraszania i bez kont dla domowników.
- **Bez synchronizacji w chmurze i obsługi wielu urządzeń** — dane żyją na jednym komputerze; domknięcie wymagania o nieopuszczaniu urządzenia przez dane.
- **Bez przypomnień docierających poza aplikację** — żadnego kanału zewnętrznego, także poza MVP; wymagałby działania w tle i wysyłki danych na zewnątrz.
- **Bez towarzyszącej aplikacji mobilnej** — telefon nie jest częścią projektu, nawet jako podgląd.
- **Bez pełnej dostępności (WCAG)** — świadomy non-goal niefunkcjonalny przy jednym, znanym użytkowniku.

## Open Questions

Brak otwartych pytań. Cztery pytania otwarte z poprzedniego wydania zostały rozstrzygnięte 2026-09-19. Zapis rozstrzygnięć zachowany poniżej, żeby uzasadnienia i odrzucone warianty nie zginęły — nie są to pytania oczekujące na decyzję.

1. **Przypomnienie docierające poza aplikację a wymaganie o nieopuszczaniu urządzenia** — ROZSTRZYGNIĘTE: pozycja usunięta z Success Criteria / Secondary i zapisana jako non-goal. Wymaganie o braku wysyłki danych na zewnątrz pozostaje zobowiązaniem binarnym, bez wyjątków i bez furtki na zgodę użytkownika. Owner: użytkownik. Data: 2026-09-19.
2. **Brak ścieżki odzyskania dostępu przy szyfrowaniu danych** — ROZSTRZYGNIĘTE: brak odzyskiwania jest właściwością produktu, nie luką; opisany w `## Access Control`. Odrzucono eksport kopii zapasowej (naruszałby szyfrowanie) oraz klucz odzyskiwania (dokładałby mechanizm do budżetu MVP). Ryzyko przeniesione na zewnętrzny menedżer haseł użytkownika. Owner: użytkownik. Data: 2026-09-19.
3. **Podstawa progu serwisu cyklicznego** — ROZSTRZYGNIĘTE: próg liczony od bieżącego stanu licznika i daty wykonania, zgodnie z FR-011. Odrzucono liczenie od planu nominalnego oraz wybór dokonywany przy każdym serwisie. Świadomie przyjęta konsekwencja — spóźniony serwis trwale przesuwa harmonogram — zapisana w `## Business Logic`. Owner: użytkownik. Data: 2026-09-19.
4. **Rozjazd między guardrailem niezawodności przypomnień a FR-010** — ROZSTRZYGNIĘTE przez dostosowanie guardraila do mechanizmu, nie odwrotnie. Guardrail mówi o niezawodności przy każdym otwarciu aplikacji i wprost nazywa nieotwarcie aplikacji ograniczeniem przyjętym świadomie. Odrzucono dokładanie sprawdzania progów niezależnego od otwarcia aplikacji. Owner: użytkownik. Data: 2026-09-19.
