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

- Faza 2 (`40b52e2`): `ServiceInput` wymaga wszystkich kluczy — brak klucza to `Err`, nie `null` (zapomniane pole nie gubi po cichu progu ani cykliczności); frontend zawsze wysyła każdy klucz, `null` dla pustego.
- Faza 2 (`40b52e2`): `update_service` zwraca `notFound` dla nieznanego `id` przed walidacją danych (UI: „Ten wpis już nie istnieje.” i odświeżenie listy).

### Weryfikacja ręczna Fazy 3 (2026-11-19)

- Kroki 3.3–3.9 wykonane przez użytkownika: `npm run tauri dev` z worktree `plan-service` na pustym katalogu danych — wszystkie przeszły.
- Prawdziwe dane odłożone przed testem do `%USERPROFILE%\kmplus-real-backup-2026-11-19` i przywrócone po nim; przywrócony katalog danych zgodny bajtowo z kopią:
  - `kmplus.vault` — 328 B, SHA-256 `88f8e1a2c65185f7ec61ff44448cf971d123d12c37690ed40508c1a1bc96c5d0` (format v2; równy wartości zapisanej po aktualizacji do 0.1.6 — punkt odniesienia dla 4.3 i porównania z kopią `pre-v3-*` w Fazie 4)
  - `kmplus.vault.bak` — SHA-256 `39af9a33a6704a974bdd3d20e53561ee2921ee6f43ff294d01918d964a47c66a`
  - `kmplus.vault.pre-v2-1792319089` — SHA-256 `43c94592a5add3a3e4e4a912768e7dd194abe70cd719d0cb5b32fb332a0143e4`
