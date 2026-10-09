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

## Vault format

| Nazwa | Gdzie zdefiniowana | Konsekwencja zmiany |
| --- | --- | --- |
| Magic `KMPV` | `src-tauri/src/vault/format.rs` (`MAGIC`) | Wszystkie istniejące pliki traktowane jako uszkodzone. |
| `envelope_version` = `1` | `src-tauri/src/vault/format.rs` (`ENVELOPE_VERSION`) | Wyższa wartość: starsze aplikacje odmawiają otwarcia (`tooNew`). Wymaga migratora i kopii przed migracją. |
| `format_version` = `1` (jawny ładunek JSON) | `src-tauri/src/vault/data.rs` (`CURRENT_FORMAT_VERSION`) | Wyższa wartość: starsze aplikacje odmawiają otwarcia (`tooNew`). Wymaga migratora i kopii przed migracją. |
| Domyślne parametry KDF Argon2id: `m_cost` 65536 KiB / `t_cost` 3 / `p_cost` 1 | `src-tauri/src/vault/format.rs` (`KdfParams::default`) | Dotyczy tylko nowo zakładanych sejfów (parametry są zapisane w nagłówku); zmienia czas odblokowania. |
| Długość nagłówka 59 B (cały nagłówek = AAD) | `src-tauri/src/vault/format.rs` (`HEADER_LEN`) | Zmiana układu nagłówka unieważnia wszystkie istniejące pliki — tylko razem z nową `envelope_version`. |

## Tauri commands

Rejestrowane w `src-tauri/src/lib.rs` (`generate_handler!`), zdefiniowane w `src-tauri/src/vault/commands.rs`. Kształty wyników są unią z polem `kind` (serde `tag = "kind"`, camelCase); czasy w milisekundach epoki. Zmiana nazwy komendy, wartości `kind` lub nazwy pola psuje frontend w czasie działania (brak kontroli przy kompilacji).

| Komenda | Argumenty | Wynik (`kind`) |
| --- | --- | --- |
| `vault_status` | — | `VaultStatus`: `needsSetup`, `locked`, `missingWithBackup` (+ `backupSavedAt`), `unlocked` |
| `vault_setup` | `password` | `SetupResult`: `ok`, `passwordTooShort`, `alreadyExists` |
| `vault_unlock` | `password` | `UnlockResult`: `unlocked`, `wrongPassword`, `damagedBackupAvailable` (+ `backupSavedAt`), `damagedNoBackup`, `tooNew` |
| `vault_restore_backup` | `password` | `UnlockResult` (jak wyżej) |

Błąd (`Err`) każdej komendy to czytelny komunikat po polsku, nigdy nie zawierający hasła.
