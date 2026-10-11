---
change_id: first-fillup-consumption
title: Punkt startowy i pierwsze tankowanie ze spalaniem i kosztem
status: implementing
created: 2026-10-11
updated: 2026-10-11
archived_at: null
---

## Notes

Roadmap: `context/foundation/roadmap.md` → S-02 (north star). PRD refs: US-01, FR-002, FR-003, FR-007. Prerequisite S-01 (`password-unlocked-vault`) is done.

Outcome: przy pierwszym uruchomieniu użytkownik podaje punkt startowy (data i przebieg ostatniego tankowania), dodaje wpis tankowania (rodzaj paliwa, data, przebieg, litry, kwota, czy do pełna) i bez dodatkowej akcji widzi spalanie tego paliwa oraz koszt.

Open unknowns from the roadmap (non-blocking):
- What does "koszt" mean in the success criterion: fill-up amount, period cost, or price per litre? Cost in zł/100 km is parked.
- An entry whose odometer is lower than the previous one, or whose date is before the starting point: reject it or warn?
- Can the starting point be edited or deleted later? Roadmap default: no, following the letter of the PRD.

Risk: pairing full-tank fill-ups of the same fuel is where a wrong number is most likely, so tests for this rule are written in this change.
