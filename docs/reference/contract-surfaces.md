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
| `kmplus.vault.tmp`, `kmplus.vault.bak.tmp` (pliki robocze zapisu atomowego) | `src-tauri/src/vault/mod.rs` (`TMP_FILE`, `BACKUP_TMP_FILE`) | Osierocone pliki robocze starszych wersji zostają w katalogu; brak utraty danych, ale niespójny katalog. |
| `kmplus.vault.damaged-<unix s>` (odłożony uszkodzony plik główny) | `src-tauri/src/vault/mod.rs` (`DAMAGED_PREFIX`) | Narzędzia i instrukcje odzyskiwania szukające tego wzorca przestają go znajdować. |
| `kmplus.vault.pre-v<N>-<unix s>` (nietknięta kopia pliku głównego w starszym formacie ładunku, zapisywana przy pierwszym zapisie po migracji do formatu `N`; nigdy nie usuwana) | `src-tauri/src/vault/mod.rs` (`PRE_MIGRATION_PREFIX`) | Narzędzia i instrukcje odzyskiwania sprzed migracji przestają znajdować kopię. |

## Vault format

| Nazwa | Gdzie zdefiniowana | Konsekwencja zmiany |
| --- | --- | --- |
| Magic `KMPV` | `src-tauri/src/vault/format.rs` (`MAGIC`) | Wszystkie istniejące pliki traktowane jako uszkodzone. |
| `envelope_version` = `1` | `src-tauri/src/vault/format.rs` (`ENVELOPE_VERSION`) | Wyższa wartość: starsze aplikacje odmawiają otwarcia (`tooNew`). Wymaga migratora i kopii przed migracją. |
| `format_version` = `2` (jawny ładunek JSON; v1 migrowany w pamięci przez `migrate_v1_to_v2`) | `src-tauri/src/vault/data.rs` (`CURRENT_FORMAT_VERSION`) | Wyższa wartość: starsze aplikacje odmawiają otwarcia (`tooNew`). Wymaga migratora i kopii przed migracją. |
| Domyślne parametry KDF Argon2id: `m_cost` 65536 KiB / `t_cost` 3 / `p_cost` 1 | `src-tauri/src/vault/format.rs` (`KdfParams::default`) | Dotyczy tylko nowo zakładanych sejfów (parametry są zapisane w nagłówku); zmienia czas odblokowania. |
| Długość nagłówka 59 B (cały nagłówek = AAD) | `src-tauri/src/vault/format.rs` (`HEADER_LEN`) | Zmiana układu nagłówka unieważnia wszystkie istniejące pliki — tylko razem z nową `envelope_version`. |

## Vault payload

Jawny ładunek JSON v2 wewnątrz sejfu (`src-tauri/src/vault/data.rs`, `VaultData`; typy domenowe w `src-tauri/src/fuel/mod.rs`). Nazwy pól są w snake_case i wszystkie są wymagane (brak pola w v2 = uszkodzenie). Zmiana nazwy pola, wartości lub jednostki unieważnia istniejące dane — tylko razem z nowym `format_version` i migratorem.

| Pole | Typ / jednostka | Uwagi |
| --- | --- | --- |
| `format_version` | liczba całkowita | `2` |
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

## Tauri commands

Rejestrowane w `src-tauri/src/lib.rs` (`generate_handler!`), zdefiniowane w `src-tauri/src/vault/commands.rs` (sejf) i `src-tauri/src/fuel/commands.rs` (paliwa). Kształty wyników są unią z polem `kind` (serde `tag = "kind"`, camelCase); czasy w milisekundach epoki; jednostki paliw jak w „Vault payload” (km, centylitry, grosze, `YYYY-MM-DD`), wartości liczone (`litresPer100Km`, `pricePerLitreGrosze`) to liczby zmiennoprzecinkowe bez zaokrąglenia. Zmiana nazwy komendy, wartości `kind` lub nazwy pola psuje frontend w czasie działania (brak kontroli przy kompilacji).

| Komenda | Argumenty | Wynik (`kind`) |
| --- | --- | --- |
| `vault_status` | — | `VaultStatus`: `needsSetup`, `locked`, `missingWithBackup` (+ `backupSavedAt`), `unlocked` |
| `vault_setup` | `password` | `SetupResult`: `ok`, `passwordTooShort`, `alreadyExists` |
| `vault_unlock` | `password` | `UnlockResult`: `unlocked`, `wrongPassword`, `damagedBackupAvailable` (+ `backupSavedAt`), `damagedNoBackup`, `tooNew` |
| `vault_restore_backup` | `password` | `UnlockResult` (jak wyżej) |
| `fuel_overview` | — | `FuelOverview` (bez `kind`): `{ startPoint: { date, odometerKm } \| null, lpg: LatestConsumption \| null, petrol: LatestConsumption \| null }`; `LatestConsumption` = `{ date, odometerKm, litresPer100Km, distanceKm, periodCostGrosze }` |
| `set_start_point` | `date`, `odometerKm` | `SetStartPointResult`: `ok`, `alreadySet`, `invalid` (+ `reason`: `"invalidDate"` \| `"odometerOutOfRange"`) |
| `add_fill_up` | `input`: `{ fuel: "lpg" \| "petrol", date, odometerKm, litresCl, amountGrosze, fullTank }` | `AddFillUpResult`: `saved` (+ `result`: `FillUpResult`), `noStartPoint`, `rejected` (+ `reason`: `FillUpRejection`) |

Typy zagnieżdżone w `add_fill_up`:

- `FillUpResult`: `consumption` (+ `litresPer100Km`, `distanceKm`, `periodLitresCl`, `periodCostGrosze`, `pricePerLitreGrosze`), `topUp` (+ `pricePerLitreGrosze`).
- `FillUpRejection`: `invalidDate`, `odometerOutOfRange`, `litresOutOfRange`, `amountOutOfRange`, `dateBeforeStart`, `odometerNotAfterStart`, `sameFuelSameOdometer` (+ `date`), `orderConflict` (+ `date`, `odometerKm`).

Błąd (`Err`) każdej komendy to czytelny komunikat po polsku, nigdy nie zawierający hasła. Komendy paliwowe bez odblokowanej sesji zwracają `Err("dane są zablokowane; odblokuj aplikację")`.
