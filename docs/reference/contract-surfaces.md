# Contract Surfaces

Rejestr nazw nośnych: wartości, których zmiana psuje istniejące dane użytkownika albo kontrakt frontend ↔ backend. Przed zmianą którejkolwiek z nich trzeba zaplanować migrację lub równoległą zmianę po drugiej stronie.

## Identity

| Nazwa | Gdzie zdefiniowana | Konsekwencja zmiany |
| --- | --- | --- |
| Identyfikator aplikacji `io.github.m-swietek.kmplus` (katalog danych `%APPDATA%\io.github.m-swietek.kmplus`) | `src-tauri/tauri.conf.json` (`identifier`) | Aplikacja szuka danych w nowym, pustym katalogu — pozorna utrata całej historii; także zerwanie ścieżki aktualizacji. |

## Vault files

| Nazwa | Gdzie zdefiniowana | Konsekwencja zmiany |
| --- | --- | --- |
| `kmplus.vault` (sejf główny) | `src-tauri/src/vault/mod.rs` (`VAULT_FILE`) | Istniejący sejf przestaje być widoczny; aplikacja proponuje ustawienie nowego hasła lub przywracanie kopii. |
| `kmplus.vault.bak` (poprzednia dobra wersja) | `src-tauri/src/vault/mod.rs` (`BACKUP_FILE`) | Kopie zapisane przez starsze wersje nie są wykrywane — brak przywracania po uszkodzeniu. |
| `kmplus.vault.tmp`, `kmplus.vault.bak.tmp`, `kmplus.vault.pre.tmp` (pliki robocze zapisu atomowego) | `src-tauri/src/vault/mod.rs` (`TMP_FILE`, `BACKUP_TMP_FILE`, `PRE_MIGRATION_TMP_FILE`) | Osierocone pliki robocze starszych wersji zostają w katalogu; brak utraty danych, ale niespójny katalog. |
| `kmplus.vault.damaged-<unix s>` (odłożony uszkodzony plik główny) | `src-tauri/src/vault/mod.rs` (`DAMAGED_PREFIX`) | Narzędzia i instrukcje odzyskiwania szukające tego wzorca przestają go znajdować. |
| `kmplus.vault.pre-v<N>-<unix s>` (nietknięta kopia pliku głównego w starszym formacie ładunku, zapisywana przy pierwszym zapisie po migracji do formatu `N`, czyli bieżącego — np. `kmplus.vault.pre-v3-<unix s>` dla pliku v1 lub v2; nigdy nie usuwana) | `src-tauri/src/vault/mod.rs` (`PRE_MIGRATION_PREFIX`) | Narzędzia i instrukcje odzyskiwania sprzed migracji przestają znajdować kopię. |

## Vault format

| Nazwa | Gdzie zdefiniowana | Konsekwencja zmiany |
| --- | --- | --- |
| Magic `KMPV` | `src-tauri/src/vault/format.rs` (`MAGIC`) | Wszystkie istniejące pliki traktowane jako uszkodzone. |
| `envelope_version` = `1` | `src-tauri/src/vault/format.rs` (`ENVELOPE_VERSION`) | Wyższa wartość: starsze aplikacje odmawiają otwarcia (`tooNew`). Wymaga migratora i kopii przed migracją. |
| `format_version` = `3` (jawny ładunek JSON; starsze migrowane w pamięci łańcuchem v1 → v2 → v3: `migrate_v1_to_v2`, `migrate_v2_to_v3`) | `src-tauri/src/vault/data.rs` (`CURRENT_FORMAT_VERSION`) | Wyższa wartość: starsze aplikacje odmawiają otwarcia (`tooNew`). Wymaga migratora i kopii przed migracją. |
| Domyślne parametry KDF Argon2id: `m_cost` 65536 KiB / `t_cost` 3 / `p_cost` 1 | `src-tauri/src/vault/format.rs` (`KdfParams::default`) | Dotyczy tylko nowo zakładanych sejfów (parametry są zapisane w nagłówku); zmienia czas odblokowania. |
| Długość nagłówka 59 B (cały nagłówek = AAD) | `src-tauri/src/vault/format.rs` (`HEADER_LEN`) | Zmiana układu nagłówka unieważnia wszystkie istniejące pliki — tylko razem z nową `envelope_version`. |

## Vault payload

Jawny ładunek JSON v3 wewnątrz sejfu (`src-tauri/src/vault/data.rs`, `VaultData`; typy domenowe w `src-tauri/src/fuel/mod.rs` i `src-tauri/src/service/mod.rs`). Nazwy pól są w snake_case i wszystkie klucze są wymagane, także te, których wartość może być `null` (brak klucza w v3 = uszkodzenie). Zmiana nazwy pola, wartości lub jednostki unieważnia istniejące dane — tylko razem z nowym `format_version` i migratorem.

| Pole | Typ / jednostka | Uwagi |
| --- | --- | --- |
| `format_version` | liczba całkowita | `3` |
| `created_at` | sekundy epoki | czas założenia sejfu |
| `start_point` | obiekt `{ date, odometer_km }` albo `null` | ustawiany raz; `date` = `YYYY-MM-DD`, `odometer_km` w km (`0..=9 999 999`) |
| `fill_ups` | tablica `FillUp` | wyniki spalania nie są zapisywane — liczone na nowo z całej listy |
| `next_fill_up_id` | liczba całkowita | id następnego tankowania, start od `1`; id nigdy nie są używane ponownie |
| `FillUp.id` | liczba całkowita | nadawane z `next_fill_up_id` |
| `FillUp.fuel` | `"lpg"` \| `"petrol"` | `Fuel` (serde lowercase) |
| `FillUp.date` | `YYYY-MM-DD` | porównywane leksykograficznie |
| `FillUp.odometer_km` | km (całkowite) | `1..=9 999 999` |
| `FillUp.litres_cl` | centylitry | `1..=100 000` |
| `FillUp.amount_grosze` | grosze | `1..=10 000 000` |
| `FillUp.full_tank` | bool | tylko `true` zamyka okres spalania |
| `services` | tablica `PlannedService` | rejestr serwisów (od v3; po migracji z v2 pusty); kolejność wyświetlania liczona przy odczycie (`register_order`), nie zapisywana |
| `next_service_id` | liczba całkowita | id następnego serwisu, start od `1`; id nigdy nie są używane ponownie |
| `PlannedService.id` | liczba całkowita | nadawane z `next_service_id` |
| `PlannedService.name` | tekst | przycięty, `1..=100` znaków Unicode |
| `PlannedService.note` | tekst albo `null` | przycięta, do 500 znaków Unicode; pusta zapisywana jako `null` |
| `PlannedService.due_odometer_km` | km (całkowite) albo `null` | próg przebiegu (wskazanie licznika), `1..=9 999 999`; co najmniej jeden z `due_odometer_km` / `due_date` jest ustawiony |
| `PlannedService.due_date` | `YYYY-MM-DD` albo `null` | termin; porównywany leksykograficznie |
| `PlannedService.recurrence` | obiekt `Recurrence` albo `null` | `null` = wpis jednorazowy |
| `Recurrence.every_km` | km (całkowite) albo `null` | interwał „co N km”, `1..=9 999 999`; ustawiony wtedy i tylko wtedy, gdy `due_odometer_km` jest ustawiony |
| `Recurrence.every_months` | miesiące (całkowite) albo `null` | interwał „co M miesięcy”, `1..=120`; ustawiony wtedy i tylko wtedy, gdy `due_date` jest ustawiony |

## Tauri commands

Rejestrowane w `src-tauri/src/lib.rs` (`generate_handler!`), zdefiniowane w `src-tauri/src/vault/commands.rs` (sejf), `src-tauri/src/fuel/commands.rs` (paliwa) i `src-tauri/src/service/commands.rs` (serwisy). Kształty wyników są unią z polem `kind` (serde `tag = "kind"`, camelCase); czasy w milisekundach epoki; jednostki paliw jak w „Vault payload” (km, centylitry, grosze, `YYYY-MM-DD`), wartości liczone (`litresPer100Km`, `pricePerLitreGrosze`) to liczby zmiennoprzecinkowe bez zaokrąglenia. Zmiana nazwy komendy, wartości `kind` lub nazwy pola psuje frontend w czasie działania (brak kontroli przy kompilacji).

| Komenda | Argumenty | Wynik (`kind`) |
| --- | --- | --- |
| `vault_status` | — | `VaultStatus`: `needsSetup`, `locked`, `missingWithBackup` (+ `backupSavedAt`), `unlocked` |
| `vault_setup` | `password` | `SetupResult`: `ok`, `passwordTooShort`, `alreadyExists` |
| `vault_unlock` | `password` | `UnlockResult`: `unlocked`, `wrongPassword`, `damagedBackupAvailable` (+ `backupSavedAt`), `damagedNoBackup`, `tooNew` |
| `vault_restore_backup` | `password` | `UnlockResult` (jak wyżej) |
| `fuel_overview` | — | `FuelOverview` (bez `kind`): `{ startPoint: { date, odometerKm } \| null, lpg: FuelView \| null, petrol: FuelView \| null }` (`lpg`/`petrol` są `null` wtedy i tylko wtedy, gdy `startPoint` jest `null`); `FuelView` = `{ summary: FuelSummaryView, history: HistoryRow[] }` (historia od najnowszego, tj. najwyższego przebiegu); `FuelSummaryView`: `noEntries`, `noClosedPeriod`, `consumption` (+ `latest`: `LatestConsumption`, `average`: `AverageConsumption`); `LatestConsumption` = `{ date, odometerKm, litresPer100Km, distanceKm, periodCostGrosze }`; `AverageConsumption` = `{ litresPer100Km, distanceKm, litresCl, periods }`; `HistoryRow` = `{ id, date, odometerKm, litresCl, amountGrosze, fullTank, result: FillUpResult }` (bez `fuel` — lista jest per paliwo). Kształt odpowiedzi jest liczony przy każdym wywołaniu i nie dotyka formatu sejfu (bez zmiany `format_version`). |
| `set_start_point` | `date`, `odometerKm` | `SetStartPointResult`: `ok`, `alreadySet`, `invalid` (+ `reason`: `{ kind: "invalidDate" }` \| `{ kind: "odometerOutOfRange" }`) |
| `add_fill_up` | `input`: `{ fuel: "lpg" \| "petrol", date, odometerKm, litresCl, amountGrosze, fullTank }` | `AddFillUpResult`: `saved` (+ `result`: `FillUpResult`), `noStartPoint`, `rejected` (+ `reason`: `FillUpRejection`) |
| `update_fill_up` | `id` (`HistoryRow.id`), `input` (jak w `add_fill_up`) | `UpdateFillUpResult`: `saved` (+ `result`: `FillUpResult` poprawionego wpisu), `notFound`, `noStartPoint`, `rejected` (+ `reason`: `FillUpRejection`; reguły dodawania wobec wszystkich wpisów poza poprawianym). Wpis zachowuje `id`, także przy zmianie paliwa. Odrzucenie i `notFound` niczego nie zapisują. |
| `delete_fill_up` | `id` (`HistoryRow.id`) | `DeleteFillUpResult`: `deleted`, `notFound`. Usunięcie jest ostateczne; usunięte `id` nie jest używane ponownie. `notFound` niczego nie zapisuje. |

`update_fill_up` i `delete_fill_up` zmieniają tylko listę `fill_ups` i nie zmieniają formatu sejfu (bez zmiany `format_version`, `next_fill_up_id` bez zmian).

Typy zagnieżdżone w `add_fill_up` i `update_fill_up` (wspólne dla obu komend):

- `FillUpResult`: `consumption` (+ `litresPer100Km`, `distanceKm`, `periodLitresCl`, `periodCostGrosze`, `pricePerLitreGrosze`), `topUp` (+ `pricePerLitreGrosze`).
- `FillUpRejection`: `invalidDate`, `odometerOutOfRange`, `litresOutOfRange`, `amountOutOfRange`, `dateBeforeStart`, `odometerNotAfterStart`, `sameFuelSameOdometer` (+ `date`), `orderConflict` (+ `date`, `odometerKm`).

Komendy serwisów (jednostki jak w „Vault payload”: km, miesiące, `YYYY-MM-DD`; nie wymagają punktu startowego; odrzucenie i `notFound` niczego nie zapisują; id usuniętego wpisu nie jest używane ponownie):

| Komenda | Argumenty | Wynik (`kind`) |
| --- | --- | --- |
| `service_register` | — | `ServiceRegister` (bez `kind`): `{ services: ServiceView[] }` w kolejności wyświetlania (`register_order`: najpierw wpisy z terminem rosnąco po dacie, potem wpisy z samym progiem przebiegu rosnąco po km; remis rozstrzyga `id`) |
| `add_service` | `input`: `ServiceInput` | `AddServiceResult`: `saved` (+ `id`), `rejected` (+ `reason`: `ServiceRejection`) |
| `update_service` | `id`, `input`: `ServiceInput` | `UpdateServiceResult`: `saved`, `notFound`, `rejected` (+ `reason`: `ServiceRejection`); zastępuje wszystkie pola wpisu, `id` bez zmian; `notFound` ma pierwszeństwo przed walidacją |
| `delete_service` | `id` | `DeleteServiceResult`: `deleted`, `notFound` |

Typy zagnieżdżone w komendach serwisów:

- `ServiceView` = `{ id, name, note: string \| null, dueOdometerKm: number \| null, dueDate: string \| null, recurrence: Recurrence \| null }`.
- `ServiceInput` = `{ name, note: string \| null, dueOdometerKm: number \| null, dueDate: string \| null, recurrence: Recurrence \| null }` — wszystkie klucze wymagane (brak klucza = `Err`), wartości mogą być `null`; `name` i `note` przycinane przed walidacją, pusta notatka zapisywana jako `null`.
- `Recurrence` = `{ everyKm: number \| null, everyMonths: number \| null }` (oba klucze wymagane).
- `ServiceRejection`: `nameEmpty`, `nameTooLong`, `noteTooLong`, `noThreshold`, `odometerOutOfRange`, `invalidDate`, `intervalMismatch`, `intervalKmOutOfRange`, `intervalMonthsOutOfRange`.

Błąd (`Err`) każdej komendy to czytelny komunikat po polsku, nigdy nie zawierający hasła. Komendy paliwowe i serwisowe bez odblokowanej sesji zwracają `Err("dane są zablokowane; odblokuj aplikację")`.
