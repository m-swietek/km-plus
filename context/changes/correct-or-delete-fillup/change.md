---
change_id: correct-or-delete-fillup
title: Poprawka i usunięcie wpisu tankowania z przeliczeniem spalania
status: implementing
created: 2026-11-08
updated: 2026-11-08
archived_at: null
---

## Notes

Źródło: `context/foundation/roadmap.md` — S-04 (`correct-or-delete-fillup`), PRD refs: US-01, FR-004, FR-005. Prerequisite S-03 (`fillup-history-both-fuels`) — wydane jako 0.1.5/0.1.6, zarchiwizowane 2026-11-08 → `context/archive/2026-10-26-fillup-history-both-fuels/` (roadmapa: `done`).

- **Outcome:** użytkownik może poprawić lub usunąć wpis tankowania, a spalanie obu paliw przelicza się ponownie.
- **Unknown (Owner: user, Block: no):** czy usunięcie wymaga potwierdzenia lub możliwości cofnięcia? — rozstrzygnięte w `/10x-plan` 2026-11-08: dwuetapowe potwierdzenie w wierszu, bez cofania i bez kosza.
- **Risk:** pierwszy destrukcyjny zapis na jedynej kopii historii; przeliczenie po edycji musi dawać ten sam wynik, co wpisanie danych od nowa.
- **Równoległa zmiana:** S-05 (`plan-service`) w osobnym worktree — wspólne pliki: `src/AppShell.tsx` (zakładki „Paliwo / Serwisy”), `src-tauri/src/lib.rs`, `docs/reference/contract-surfaces.md`, pliki wersji. Ręczna weryfikacja Fazy 3 i wydania nie mogą nakładać się w czasie z S-05 (wspólny katalog danych, port 1420).
