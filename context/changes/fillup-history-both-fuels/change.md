---
change_id: fillup-history-both-fuels
title: Historia tankowań i bieżące spalanie osobno dla gazu i benzyny
status: implementing
created: 2026-10-26
updated: 2026-11-08
archived_at: null
---

## Notes

Źródło: `context/foundation/roadmap.md` — S-03 (`fillup-history-both-fuels`), PRD refs: US-01, FR-007. Prerequisite S-02 (`first-fillup-consumption`) — done.

- **Outcome:** użytkownik może przeglądać historię tankowań oraz bieżące spalanie osobno dla gazu i dla benzyny, także przy przeplatanych tankowaniach obu paliw i częściowych dolewkach.
- **Unknown (Owner: user, Block: no):** jak zakomunikować brak wyniku dla paliwa, które nie ma jeszcze zamkniętego okresu (np. same dolewki), skoro nie wolno pokazać szacunku?
- **Risk:** przeplatanie paliw i dolewki to scenariusze, które ujawniają błędy reguły spalania z S-02.

## Dowód wydania 0.1.5 (Faza 4)

- Run `release.yml` 42501063929 (2026-11-08, commit `30b1eed`): sukces w 10m17s, krok „Rust tests” zielony; draft `app-v0.1.5` z `km-plus_0.1.5_x64-setup.exe` (2 332 463 B, SHA-256 `bc84dd05d1ddeb471dd3dddb8131500c149580fbcf13487dda4dd239e893e00b`), `.sig` i `latest.json` (wersja 0.1.5, podpis zgodny z `.sig`). Opublikowany ręcznie przez użytkownika 2026-11-08 16:53:49 UTC; publiczny `releases/latest/download/latest.json` serwuje 0.1.5.
- Przed aktualizacją (zainstalowana 0.1.4): `kmplus.vault` 328 B, mtime 2026-10-18 12:25:04, SHA-256 `88f8e1a2c65185f7ec61ff44448cf971d123d12c37690ed40508c1a1bc96c5d0`.
- Po aktualizacji 0.1.4 → 0.1.5 przez baner i odblokowaniu tym samym hasłem bez zapisu: `kmplus.vault` SHA-256 `88f8e1a2…c5d0` (identyczny), mtime bez zmian; brak nowego pliku `pre-v*` (w katalogu tylko `kmplus.vault`, `kmplus.vault.bak` i istniejący `kmplus.vault.pre-v2-1792319089`). Zainstalowana wersja w rejestrze deinstalacji: 0.1.5 (`%LOCALAPPDATA%\km-plus`).
- Istniejące tankowania widoczne w listach historii z wynikami, karty pokazują ostatni okres i średnią (potwierdzenie użytkownika, 2026-11-08).
- Przed wydaniem w katalogu danych nadal leżał testowy sejf z ręcznego testu Fazy 3 (683 B); prawdziwe pliki leżały w `Downloads\backup` (SHA-256 zgodne z dowodem S-02). 2026-11-08 testowy katalog przemianowany na `io.github.m-swietek.kmplus.test-s03`, prawdziwe pliki skopiowane z powrotem (hashe zweryfikowane) — dopiero potem zanotowano stan sprzed aktualizacji.
