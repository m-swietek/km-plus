---
change_id: plan-service
title: Rejestr serwisów — planowanie, edycja i usuwanie przeglądu lub wymiany
status: implementing
created: 2026-11-08
updated: 2026-11-17
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
