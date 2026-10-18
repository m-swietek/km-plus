---
change_id: first-fillup-consumption
title: Punkt startowy i pierwsze tankowanie ze spalaniem i kosztem
status: archived
created: 2026-10-11
updated: 2026-10-18
archived_at: 2026-10-18T10:41:09Z
---

## Notes

Roadmap: `context/foundation/roadmap.md` → S-02 (north star). PRD refs: US-01, FR-002, FR-003, FR-007. Prerequisite S-01 (`password-unlocked-vault`) is done.

Outcome: przy pierwszym uruchomieniu użytkownik podaje punkt startowy (data i przebieg ostatniego tankowania), dodaje wpis tankowania (rodzaj paliwa, data, przebieg, litry, kwota, czy do pełna) i bez dodatkowej akcji widzi spalanie tego paliwa oraz koszt.

Open unknowns from the roadmap (non-blocking):
- What does "koszt" mean in the success criterion: fill-up amount, period cost, or price per litre? Cost in zł/100 km is parked.
- An entry whose odometer is lower than the previous one, or whose date is before the starting point: reject it or warn?
- Can the starting point be edited or deleted later? Roadmap default: no, following the letter of the PRD.

Risk: pairing full-tank fill-ups of the same fuel is where a wrong number is most likely, so tests for this rule are written in this change.

## Dowód wydania i migracji (Faza 4)

- Run `release.yml` 38215438069 (2026-10-12): sukces, krok „Rust tests” zielony; wydanie `app-v0.1.4` z `km-plus_0.1.4_x64-setup.exe`, `.sig`, `latest.json`.
- Zainstalowana 0.1.4 (`%LOCALAPPDATA%\km-plus`), katalog danych `%APPDATA%\io.github.m-swietek.kmplus\`.
- `kmplus.vault` v1 po aktualizacji i odblokowaniu bez zapisu: 119 B, mtime 2026-10-12 08:44, SHA-256 `43c94592a5add3a3e4e4a912768e7dd194abe70cd719d0cb5b32fb332a0143e4` (plik niezmieniony od 12.10 do pierwszego zapisu).
- 2026-10-18 12:24:49, zapis punktu startowego: powstał `kmplus.vault.pre-v2-1792319089`, 119 B, SHA-256 `43c94592…0143e4` — identyczny z plikiem v1.
- 2026-10-18 12:25:04, zapis tankowania: `kmplus.vault` 328 B (`88f8e1a2…c5d0`), `kmplus.vault.bak` 210 B (`39af9a33…c66a`); brak drugiej kopii `pre-v2-*`.
- 2026-10-18: po restarcie i odblokowaniu wynik tankowania nadal widoczny (potwierdzenie użytkownika).
- Pierwsze ręczne potwierdzenie 4.3–4.5 (2026-10-12) nie zostawiło śladu w katalogu danych; dowód powtórzony 2026-10-18 po przeglądzie (`reviews/impl-review.md`, F1).
