---
change_id: correct-or-delete-fillup
title: Poprawka i usunięcie wpisu tankowania z przeliczeniem spalania
status: implementing
created: 2026-11-08
updated: 2026-11-19
archived_at: null
---

## Notes

Źródło: `context/foundation/roadmap.md` — S-04 (`correct-or-delete-fillup`), PRD refs: US-01, FR-004, FR-005. Prerequisite S-03 (`fillup-history-both-fuels`) — wydane jako 0.1.5/0.1.6, zarchiwizowane 2026-11-08 → `context/archive/2026-10-26-fillup-history-both-fuels/` (roadmapa: `done`).

- **Outcome:** użytkownik może poprawić lub usunąć wpis tankowania, a spalanie obu paliw przelicza się ponownie.
- **Unknown (Owner: user, Block: no):** czy usunięcie wymaga potwierdzenia lub możliwości cofnięcia? — rozstrzygnięte w `/10x-plan` 2026-11-08: dwuetapowe potwierdzenie w wierszu, bez cofania i bez kosza.
- **Risk:** pierwszy destrukcyjny zapis na jedynej kopii historii; przeliczenie po edycji musi dawać ten sam wynik, co wpisanie danych od nowa.
- **Równoległa zmiana:** S-05 (`plan-service`) w osobnym worktree — wspólne pliki: `src/AppShell.tsx` (zakładki „Paliwo / Serwisy”), `src-tauri/src/lib.rs`, `docs/reference/contract-surfaces.md`, pliki wersji. Ręczna weryfikacja Fazy 3 i wydania nie mogą nakładać się w czasie z S-05 (wspólny katalog danych, port 1420).
- **Faza 3 — weryfikacja ręczna (2026-11-19):** kroki 3.3–3.10 wykonane przez użytkownika na pustym katalogu danych (`npm run tauri dev` z worktree `correct-or-delete-fillup`), wszystkie zaliczone. Prawdziwe dane odłożone do `%USERPROFILE%\kmplus-real-backup-2026-11-19` i przywrócone; przywrócony katalog zgodny bajt w bajt z kopią: `kmplus.vault` 328 B, SHA-256 `88f8e1a2c65185f7ec61ff44448cf971d123d12c37690ed40508c1a1bc96c5d0` (równy wartości zapisanej po aktualizacji do 0.1.6), `kmplus.vault.bak` SHA-256 `39af9a33…c66a`, `kmplus.vault.pre-v2-1792319089` SHA-256 `43c94592…0143e4`. To jest dowód odniesienia dla kontroli 4.3 (o ile S-05 nie zostanie wydane wcześniej).
