---
change_id: fillup-history-both-fuels
title: Historia tankowań i bieżące spalanie osobno dla gazu i benzyny
status: implementing
created: 2026-10-26
updated: 2026-10-26
archived_at: null
---

## Notes

Źródło: `context/foundation/roadmap.md` — S-03 (`fillup-history-both-fuels`), PRD refs: US-01, FR-007. Prerequisite S-02 (`first-fillup-consumption`) — done.

- **Outcome:** użytkownik może przeglądać historię tankowań oraz bieżące spalanie osobno dla gazu i dla benzyny, także przy przeplatanych tankowaniach obu paliw i częściowych dolewkach.
- **Unknown (Owner: user, Block: no):** jak zakomunikować brak wyniku dla paliwa, które nie ma jeszcze zamkniętego okresu (np. same dolewki), skoro nie wolno pokazać szacunku?
- **Risk:** przeplatanie paliw i dolewki to scenariusze, które ujawniają błędy reguły spalania z S-02.
