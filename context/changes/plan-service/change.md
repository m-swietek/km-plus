---
change_id: plan-service
title: Rejestr serwisów — planowanie, edycja i usuwanie przeglądu lub wymiany
status: implementing
created: 2026-11-08
updated: 2026-11-19
archived_at: null
---

## Notes

Źródło: `context/foundation/roadmap.md` — S-05 (`plan-service`), PRD refs: US-02, FR-008, FR-009. Prerequisite S-01 (`password-unlocked-vault`) — done.

- **Outcome:** użytkownik może dodać zaplanowany przegląd lub wymianę z progiem przebiegu, progiem daty albo obydwoma, oznaczyć go jako cykliczny, a potem edytować lub usunąć w rejestrze serwisów.
- **Risk:** oznaczenie „cykliczny” samo nie tworzy kolejnego wystąpienia (US-02) — to S-08; przypomnienia to S-07.
- Planowane równolegle z S-04 (`correct-or-delete-fillup`, osobny worktree) — punkty kolizji przy scaleniu opisane w `plan.md` (Open Risks w `plan-brief.md`).

### Decyzje z implementacji (potwierdzone przez użytkownika)

- Faza 2 (`fe32325`): `ServiceInput` wymaga wszystkich kluczy — brak klucza to `Err`, nie `null` (zapomniane pole nie gubi po cichu progu ani cykliczności); frontend zawsze wysyła każdy klucz, `null` dla pustego.
- Faza 2 (`fe32325`): `update_service` zwraca `notFound` dla nieznanego `id` przed walidacją danych (UI: „Ten wpis już nie istnieje.” i odświeżenie listy).

### Weryfikacja ręczna Fazy 3 (2026-11-19)

- Kroki 3.3–3.9 wykonane przez użytkownika: `npm run tauri dev` z worktree `plan-service` na pustym katalogu danych — wszystkie przeszły.
- Prawdziwe dane odłożone przed testem do `%USERPROFILE%\kmplus-real-backup-2026-11-19` i przywrócone po nim; przywrócony katalog danych zgodny bajtowo z kopią:
  - `kmplus.vault` — 328 B, SHA-256 `88f8e1a2c65185f7ec61ff44448cf971d123d12c37690ed40508c1a1bc96c5d0` (format v2; równy wartości zapisanej po aktualizacji do 0.1.6; jako punkt odniesienia dla Fazy 4 zastąpiony — patrz niżej)
  - `kmplus.vault.bak` — SHA-256 `39af9a33a6704a974bdd3d20e53561ee2921ee6f43ff294d01918d964a47c66a`
  - `kmplus.vault.pre-v2-1792319089` — SHA-256 `43c94592a5add3a3e4e4a912768e7dd194abe70cd719d0cb5b32fb332a0143e4`

### Punkt odniesienia dla Fazy 4 (po rebase na `main`, 2026-11-19)

- S-04 wydane jako `app-v0.1.7` (Latest); gałąź `feature/plan-service` przeniesiona na `main` (`ccf70a2`). `main` ma nadal `CURRENT_FORMAT_VERSION = 2` (S-04 nie zmienił formatu ani `src-tauri/src/vault/`), więc migracja v2 → v3 pozostaje bez zmian.
- Sejf sprzed aktualizacji na prawdziwych danych (po teście S-04 na 0.1.7, zapisany w `context/changes/correct-or-delete-fillup/change.md` i `context/deployment/deploy-plan.md`): `kmplus.vault` — 328 B, SHA-256 `8bb0a913ca5fd3ef2fcc842b2d1643a30aeaa1566a150b8c41c9666852caa851`; zastępuje `88f8e1a2…96c5d0`. Kopia `kmplus.vault.pre-v3-*` po pierwszym zapisie w Fazie 4 musi mieć ten SHA-256 (4.3–4.5).
- Planowana wersja: **0.1.8** (`gh release list`: najnowsze `app-v0.1.7`).
- SHA w Progress przemapowane po rebase: `c475eca` → `6a6aec0`, `40b52e2` → `fe32325`, `b4aedb2` → `685d2d2`, `a17de95` → `a614f07`.
