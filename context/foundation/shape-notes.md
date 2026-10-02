---
project: "kmPlus"
context_type: greenfield
product_type: desktop
target_scale:
  users: small
timeline_budget:
  mvp_weeks: 6
  hard_deadline: 2026-12-06
  after_hours_only: true
created: 2026-09-19
updated: 2026-09-19
checkpoint:
  current_phase: 8
  phases_completed: [1, 2, 3, 4, 5, 6, 7]
  gray_areas_resolved:
    - topic: "kategoria bólu"
      decision: "dane rozproszone i niewiarygodne (komputer pokładowy przekłamuje, Excel ręczny, kalendarz osobno) + tarcie procesu przy każdym tankowaniu"
    - topic: "insight / dlaczego nie gotowa aplikacja"
      decision: "rynek nie był badany; projekt powstaje głównie jako projekt do nauki — nie zakłada się przewagi nad gotowymi narzędziami, poprzeczką jest własny Excel"
    - topic: "co znaczy faktyczne spalanie przy dwóch paliwach"
      decision: "per paliwo, atrybucja automatyczna — każdy wpis tankowania niesie rodzaj paliwa, spalanie liczone między kolejnymi tankowaniami tego samego paliwa; świadomie akceptowane zawyżenie gazu o kilometry przejechane na benzynie (błąd stały, więc trend pozostaje wiarygodny)"
    - topic: "model dostępu"
      decision: "hasło przy starcie aplikacji, jedno predefiniowane konto bez rejestracji, model płaski bez ról; hasło chroni przed przypadkowym wglądem na współdzielonym komputerze, nie przed atakiem"
    - topic: "zakres MVP"
      decision: "MVP obejmuje obie połowy — paliwo i przypomnienia serwisowe; pierwsze uruchomienie prosi o punkt startowy (data i przebieg ostatniego tankowania), żeby pierwszy wpis od razu dawał spalanie"
    - topic: "budżet czasu"
      decision: "4–6 tygodni po godzinach (zapisane 6); koszt dłuższego terminu przedstawiony wraz z konkretnymi ruchami cięcia zakresu i świadomie przyjęty zamiast cięcia"
    - topic: "kanał przypomnień"
      decision: "wyłącznie przypomnienie w interfejsie przy otwarciu aplikacji; kanały zewnętrzne odrzucone całkowicie, także poza MVP, bo wymagałyby procesu w tle i wysyłki danych na zewnątrz; guardrail sformułowany pod ten mechanizm — niezawodność dotyczy każdego otwarcia aplikacji, a nieotwarcie aplikacji jest ograniczeniem przyjętym świadomie"
    - topic: "edycja i usuwanie"
      decision: "wpisy tankowania oraz zaplanowane serwisy można edytować i usuwać; samodzielny wpis przebiegu dostępny niezależnie od tankowania"
    - topic: "próg serwisowy"
      decision: "próg przebiegu, próg daty lub oba naraz; pierwsze osiągnięte kryterium wyzwala przypomnienie; oznaczenie jako wykonany tworzy kolejne wystąpienie dla wpisów cyklicznych, z progiem liczonym od bieżącego stanu licznika i daty wykonania, a nie od progu nominalnego — dryf harmonogramu wobec planu producenta przyjęty świadomie"
    - topic: "szyfrowanie danych"
      decision: "dane przechowywane w postaci zaszyfrowanej, hasło jest warunkiem odszyfrowania (FR-001); nie istnieje żadna ścieżka odzyskania dostępu — odrzucono zarówno eksport kopii zapasowej, jak i klucz odzyskiwania; ryzyko utraty historii przeniesione na zewnętrzny menedżer haseł, opisane w Access Control jako właściwość produktu"
    - topic: "tankowanie do pełna"
      decision: "wpis tankowania niesie znacznik tankowania do pełna (FR-003); tylko takie wpisy zamykają okres rozliczeniowy spalania, częściowe dolewki nie generują wyniku"
    - topic: "reguła domenowa"
      decision: "jedna reguła o dwóch częściach — wyliczanie zużycia osobno per paliwo z parowania tankowań do pełna oraz decydowanie, kiedy ostrzec o serwisie na podstawie przebiegu lub daty; anty-wzorzec pustego CRUD nie występuje"
    - topic: "właściwości jakościowe"
      decision: "dane nieczytelne bez hasła, pełne działanie offline, brak wysyłki danych na zewnątrz; ostatni punkt jest zobowiązaniem binarnym — bez wyjątków i bez furtki na zgodę użytkownika"
    - topic: "ramy produktu"
      decision: "aplikacja desktopowa na Windows, skala small (jeden użytkownik, jedno auto), twardy termin 2026-12-06 przy pracy wyłącznie po godzinach; około 11 tygodni kalendarza wobec 6 tygodni szacunku"
    - topic: "zakres persony"
      decision: "jedna osoba, jeden pojazd — bez współdzielenia, bez ról, bez obsługi wielu aut"
  frs_drafted: 11
  quality_check_status: accepted
---

# Shape Notes

## Vision & Problem Statement

Kierowca samochodu z instalacją gazową nie zna faktycznego spalania swojego auta — komputer pokładowy podaje przekłamany wynik, bo nie rozlicza poprawnie jazdy na dwóch paliwach. Żeby mieć świadomość realnego zużycia, właściciel utrzymuje arkusz Excela i po każdym tankowaniu przepisuje do niego dane z paragonu. Równolegle terminy przeglądów i wymian części żyją w kalendarzu albo w aplikacji typu todo-lista, czyli poza kontekstem samochodu. Kosztem jest ręczna praca przy każdym tankowaniu oraz brak jednego miejsca, w którym widać stan auta — prawda o samochodzie leży rozbita na trzy źródła, z których żadne nie jest ani kompletne, ani wiarygodne.

Autor nie porównywał gotowych aplikacji do śledzenia tankowań pod kątem tego, czego im brakuje — projekt powstaje przede wszystkim jako projekt do nauki, czyli budowa własnej aplikacji desktopowej od zera. Nie zakłada się więc przewagi nad istniejącymi narzędziami rynkowymi. Punktem odniesienia, który aplikacja musi przebić, jest dotychczasowy arkusz Excela autora: ma dawać ten sam obraz spalania przy mniejszym nakładzie pracy i dokładać to, czego Excel nie robi — aktywny nadzór nad nadchodzącymi przeglądami i wymianami.

## User & Persona

Jedna osoba: autor, kierowca i właściciel jednego samochodu z instalacją gazową. Sięga po aplikację w dwóch momentach:

1. **Po tankowaniu** — żeby wpisać dane z paragonu i zobaczyć, czy spalanie się zmieniło. Dziś ten moment kończy się w Excelu.
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
- Dane nigdy nie giną. Po porzuceniu Excela aplikacja jest jedynym źródłem historii tankowań i serwisów — utrata tej historii jest awarią krytyczną, bo nie da się jej odtworzyć.
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
  > Socrates: Kontrargument rozważony: "hasło nie chroni tego, co się wydaje — kto ma konto Windows, ten i tak odczyta plik z dysku". Rozwiązanie: FR wzmocniony — dane mają być przechowywane w postaci zaszyfrowanej, a hasło jest warunkiem ich odszyfrowania, nie tylko zasłoną na ekranie. Konsekwencja przyjęta świadomie: brak odzyskiwania hasła oznacza bezpowrotną utratę historii (rozstrzygnięte 2026-09-19, patrz Access Control).
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
- Dane o przebiegu, kosztach i terminach serwisowych nie są wysyłane do żadnej zewnętrznej usługi.

## Business Logic

Aplikacja wylicza faktyczne zużycie paliwa osobno dla każdego paliwa i sama decyduje, kiedy ostrzec o zbliżającym się serwisie.

Pierwsza część reguły konsumuje wpisy tankowań — rodzaj paliwa, datę, stan licznika, liczbę litrów, kwotę oraz informację, czy tankowano do pełna — a także punkt startowy podany przy pierwszym uruchomieniu. Reguła zestawia ze sobą kolejne tankowania do pełna tego samego paliwa i z dystansu między nimi oraz zatankowanej ilości wyprowadza zużycie. Wyjściem są dwie osobne wartości: zużycie gazu i zużycie benzyny. Przyjęte świadomie uproszczenie: kilometry przejechane na benzynie przy rozruchu wliczają się do dystansu przypisanego gazowi, więc zużycie gazu jest nieznacznie zawyżone. Błąd jest systematyczny, więc porównywanie okresów między sobą pozostaje wiarygodne — i to porównanie, a nie wartość bezwzględna, jest tym, czego użytkownik szuka.

Druga część reguły konsumuje najnowszy znany stan licznika — pochodzący z tankowania albo z samodzielnego wpisu przebiegu — bieżącą datę oraz progi zdefiniowane dla zaplanowanego serwisu, którymi mogą być przebieg, data albo oba naraz. Reguła wyzwala przypomnienie, gdy pierwsze z zadanych kryteriów zostanie osiągnięte lub przekroczone; nie wymaga to spełnienia obu. Wpis pozostaje w stanie wymagającym uwagi aż do oznaczenia go jako wykonany. Dla serwisów oznaczonych jako cykliczne oznaczenie wykonania tworzy kolejne wystąpienie z progiem wyliczonym od bieżącego stanu licznika i bieżącej daty — a nie od progu pierwotnie zaplanowanego. Rozstrzygnięte świadomie: reguła odzwierciedla realny odstęp między wykonanymi serwisami, a konsekwencją jest to, że każde spóźnienie trwale przesuwa harmonogram względem nominalnego planu producenta.

Użytkownik spotyka pierwszą część reguły zaraz po zapisaniu wpisu tankowania — wynik pojawia się bez dodatkowej akcji z jego strony. Drugą spotyka przy otwarciu aplikacji, gdzie aktualne przypomnienia są widoczne od razu, bez wchodzenia w osobny widok.

## Access Control

Hasło podawane przy starcie aplikacji. Jedno predefiniowane konto właściciela — bez rejestracji i bez self-service. Model płaski: brak ról i poziomów dostępu, bo użytkownik jest dokładnie jeden.

Hasło nie jest samą zasłoną na ekranie: dane przechowywane są w postaci zaszyfrowanej, a ich postać jawna powstaje dopiero po podaniu poprawnego hasła.

Nie istnieje żadna ścieżka odzyskania dostępu bez hasła — ani pytanie pomocnicze, ani klucz zapasowy, ani eksport awaryjny. Jest to rozstrzygnięta właściwość produktu, nie luka: utrata hasła oznacza bezpowrotną utratę całej historii. Ryzyko zostało świadomie przeniesione poza aplikację, na zewnętrzny menedżer haseł użytkownika.

Hasło chroni dane przed przypadkowym wglądem osoby korzystającej z tego samego komputera. Nie jest mechanizmem rozdzielania wielu użytkowników aplikacji ani ochroną przed atakiem — taki poziom nie jest tu celem.

## Non-Goals

- **Bez wbudowanej bazy harmonogramów serwisowych** — aplikacja nie wie, co i co ile kilometrów wymienia się w danym modelu auta; progi wpisuje użytkownik.
- **Bez odczytu z komputera pokładowego (OBD)** — stan licznika zawsze wprowadzany ręcznie, bez czytnika i bez integracji z autem.
- **Bez obsługi wielu pojazdów** — jedno auto na stałe, żadnego wyboru pojazdu w interfejsie.
- **Bez współdzielenia i wielu użytkowników** — jedno konto właściciela, bez zapraszania i bez kont dla domowników.
- **Bez synchronizacji w chmurze i obsługi wielu urządzeń** — dane żyją na jednym komputerze; domknięcie NFR o nieopuszczaniu urządzenia.
- **Bez przypomnień docierających poza aplikację** — żaden kanał zewnętrzny, także poza MVP; wymagałby działania w tle i wysyłki danych na zewnątrz.
- **Bez towarzyszącej aplikacji mobilnej** — telefon nie jest częścią projektu, nawet jako podgląd.
- **Bez pełnej dostępności (WCAG)** — świadomy non-goal niefunkcjonalny przy jednym, znanym użytkowniku.

## Open Questions

Brak otwartych pytań. Wszystkie cztery, otwarte podczas shapingu i przeniesione do PRD, zostały rozstrzygnięte 2026-09-19 w osobnej rundzie domykania. Zapis rozstrzygnięć poniżej — dla zachowania uzasadnień, nie jako pytania.

1. **Przypomnienie docierające poza aplikację a wymaganie o nieopuszczaniu urządzenia** — rozstrzygnięte: pozycja usunięta z Success Criteria / Secondary. Wymaganie o braku wysyłki danych na zewnątrz pozostaje zobowiązaniem binarnym, bez wyjątków i bez furtki na zgodę użytkownika.
2. **Brak ścieżki odzyskania dostępu przy szyfrowaniu danych** — rozstrzygnięte: brak odzyskiwania jest właściwością produktu, nie luką. Przeniesione z pytań otwartych do `## Access Control`. Odrzucono eksport kopii zapasowej (robiłby dziurę w szyfrowaniu) oraz klucz odzyskiwania (dokładałby ekran i mechanizm do budżetu 6 tygodni).
3. **Podstawa progu serwisu cyklicznego** — rozstrzygnięte: próg liczony od bieżącego stanu licznika i daty wykonania, zgodnie z dotychczasowym brzmieniem FR-011. Odrzucono liczenie od nominalnego planu oraz wybór per serwis. Świadomie przyjęta konsekwencja: spóźniony serwis trwale przesuwa harmonogram względem planu producenta — zapisana wprost w `## Business Logic`.
4. **Guardrail niezawodności przypomnień wobec FR-010** — rozstrzygnięte przez dostosowanie guardraila do mechanizmu, nie odwrotnie. Guardrail mówi teraz o niezawodności przy każdym otwarciu aplikacji i wprost nazywa nieotwarcie aplikacji ograniczeniem przyjętym świadomie. Odrzucono dokładanie sprawdzania progów niezależnego od otwarcia aplikacji.

## Timeline acknowledgment

Acknowledged on 2026-09-19: MVP szacowane na 4–6 tygodni (zapisane jako 6) pracy po godzinach wymaga stałego zaangażowania przez kilkanaście wieczorów, także w okresach, gdy postęp jest niewidoczny; użytkownik świadomie zaakceptował ten koszt po przedstawieniu drogi alternatywnej (cięcie zakresu).

## Quality cross-check

Przeprowadzona 2026-09-19. Wynik: 6/6 elementów obecnych, brak luk formalnych.

- Access Control — present: hasło przy starcie plus szyfrowanie danych w spoczynku, model płaski.
- Business Logic — present: jedno zdanie o dwóch częściach, realna reguła domenowa, anty-wzorzec pustego CRUD nie występuje.
- Project artifacts — present: shape-notes.md z poprawnym blokiem checkpoint.
- Timeline-cost ack — present: MVP szacowane na 6 tygodni przekracza próg 3 tygodni, koszt przedstawiony i świadomie przyjęty (patrz `## Timeline acknowledgment`).
- Non-Goals — present: osiem pozycji, funkcjonalnych i niefunkcjonalnych (dziewiąta, dotycząca docelowego systemu operacyjnego, przeniesiona do `## Forward: tech-stack`).
- Preserved behavior — n/a: sesja greenfield.

Napięcia odnotowane mimo braku luk formalnych — oba **rozstrzygnięte 2026-09-19** w rundzie domykania pytań otwartych:

- Guardrail o niezawodności przypomnień wobec kanału ograniczonego do interfejsu aplikacji (FR-010) — domknięte przez przeformułowanie guardraila.
- Wymaganie o nieopuszczaniu urządzenia przez dane wobec przypomnienia docierającego poza aplikację — domknięte przez usunięcie tej pozycji z Secondary.


## Forward: tech-stack

Treści zebrane podczas shapingu, które nie należą do PRD i są przeznaczone dla kroku wyboru stosu technologicznego.

- **Docelowy system operacyjny: Windows.** Aplikacja celuje w jedną platformę desktopową; brak wsparcia dla macOS i Linuksa. Decyzja podjęta świadomie jako ograniczenie zakresu testowania i wykorzystania możliwości jednego systemu. Przeniesione tutaj z `## Non-Goals` podczas generowania PRD, ponieważ wybór platformy docelowej należy do kroku wyboru stosu, nie do PRD.
- **Przechowywanie danych musi umożliwiać szyfrowanie w spoczynku** z hasłem użytkownika jako warunkiem odszyfrowania (patrz FR-001 i `## Open Questions`). Do zweryfikowania przy wyborze sposobu składowania danych.
