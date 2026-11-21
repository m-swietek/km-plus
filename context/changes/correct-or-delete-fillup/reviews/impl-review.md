<!-- IMPL-REVIEW-REPORT -->
# Implementation Review: Poprawka i usunięcie tankowania

- **Plan**: context/changes/correct-or-delete-fillup/plan.md
- **Scope**: Full plan
- **Reviewed phases**: 1, 2, 3, 4
- **Date**: 2026-11-19
- **Verdict**: NEEDS ATTENTION
- **Findings**: 0 critical, 1 warning, 5 observations

## Verdicts

| Dimension | Verdict |
|-----------|---------|
| Plan Adherence | PASS |
| Scope Discipline | PASS |
| Safety & Quality | WARNING |
| Architecture | PASS |
| Pattern Consistency | WARNING |
| Success Criteria | WARNING |

Reviewed range `5a23c48..ccf70a2`: 22 changed files, all named in the plan, none missing. Automated criteria re-run at review time: `cargo test fuel::tests` 28 passed, `cargo test fuel::commands` 16 passed, `cargo test` 77 passed, `cargo check` 0 warnings, `npm run build` OK, release run 44519146885 success incl. "Rust tests", `app-v0.1.7` published with installer, `.sig`, `latest.json`. Manual rows 3.3–3.10 and 4.3–4.5 ticked with evidence in `change.md`. Roadmap S-04 still `in-progress` — expected until `/10x-archive`.

## Findings

### F1 — "Edytuj" during an in-flight add overwrites the edited entry's fields

- **Severity**: ⚠️ WARNING
- **Impact**: 🔎 MEDIUM — real tradeoff; pause to reason through it
- **Dimension**: Safety & Quality
- **Location**: src/fuel/FillUpForm.tsx:165-170, src/fuel/FillUpHistory.tsx:123-126
- **Detail**: The add `saved` branch clears odometer/litres/amount and sets fullTank=true without checking the form is still in add mode, and the history buttons were disabled only by their own list's `busy`. Click "Zapisz tankowanie", then "Edytuj" on an LPG top-up before the save returns: the form shows "Popraw tankowanie" for that row with empty fields and "do pełna" ticked; after retyping and saving, the top-up becomes a full tank and closes a period it shouldn't — a wrong consumption figure. Variant: an add rejection appears on the edit form. The plan (Phase 3 §4: "przyciski nieaktywne w trakcie żądania") was implemented per component, not screen-wide.
- **Fix A ⭐ Recommended**: Shared in-flight flag in `useFillUpEditing` (form save + either list's delete); disable "Edytuj"/"Usuń" in both lists and the form submit while set.
  - Strength: Closes F1–F3 at the source; matches the plan's intent.
  - Tradeoff: One more prop through `AppShell` (S-05 merge touches it).
  - Confidence: HIGH — the hook already holds the shared state.
  - Blind spot: Disabled-button look in dark mode not checked.
- **Fix B**: After each `await` in FillUpForm, apply the result only if still in the submitted mode (ref).
  - Strength: Local to FillUpForm.
  - Tradeoff: Leaves F2/F3 open; drops late results silently.
  - Confidence: MED — easy to miss a branch.
  - Blind spot: Notice from the abandoned add still shown.
- **Decision**: FIXED via Fix A — `Requests { inFlight, track }` in `useFillUpEditing.ts`; the form's add/update and both lists' delete go through `requests.track`; the form submit and every row's "Edytuj"/"Usuń"/confirm buttons are disabled while `inFlight`; `AppShell.tsx` passes one new prop `requests`. Not released on its own — ships with S-05's 0.1.8.

### F2 — A finished correction ends or mislabels an edit started meanwhile

- **Severity**: OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Safety & Quality
- **Location**: src/fuel/useFillUpEditing.ts:52-55, src/fuel/FillUpForm.tsx:140-160
- **Detail**: `reportUpdated` cleared `editing` unconditionally; saving A then clicking "Edytuj" on B mid-request dropped B's edit (or showed A's rejection / lost A's notFound message on B). No wrong data written.
- **Fix**: `reportUpdated` takes the id and clears only a matching edit, as `reportGone` does (plus F1 Fix A).
- **Decision**: FIXED — `reportUpdated(id, fuel, date, result)` with `setEditing(c => c?.row.id === id ? null : c)`; the form passes `editing.row.id`.

### F3 — A finished delete closes a confirmation opened on another row

- **Severity**: OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Safety & Quality
- **Location**: src/fuel/useFillUpEditing.ts:60
- **Detail**: `reportDeleted` reset `confirmingId` unconditionally while `busy` was per list; a confirmation opened in the other list mid-delete disappeared, and a failed delete's error stayed hidden. Nothing deleted wrongly.
- **Fix**: `setConfirmingId(c => c === row.id ? null : c)` (plus F1 Fix A).
- **Decision**: FIXED — functional update in `reportDeleted`; with F1 Fix A no second confirmation can be opened during a delete.

### F4 — Id-reuse test can't tell a counter from max(id)+1

- **Severity**: OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Safety & Quality
- **Location**: src-tauri/src/fuel/commands.rs:1162-1207
- **Detail**: The test deleted id 1 (leaving [2]) and expected the next add to get 3 — a max(id)+1 allocator passes too. Code is correct (`next_fill_up_id`), the test was weak.
- **Fix**: Also delete the highest id (2, leaving [1]) and assert the next add gets 3, not 2.
- **Decision**: FIXED — new test `deleting_the_highest_id_does_not_free_it` (ids [1] → add → [1, 3], `next_fill_up_id` 4, same after reopen).

### F5 — 4.3: no separate listing taken on 0.1.6 just before the update

- **Severity**: OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Success Criteria
- **Location**: context/changes/correct-or-delete-fillup/change.md (## Notes, Faza 4)
- **Detail**: The plan wanted the vault SHA-256 noted on the installed version right before the update; the evidence uses the Phase 3 restore hash plus an unchanged LastWriteTime (18.10.2026 12:25:04) — equivalent, and disclosed in `change.md`.
- **Fix**: None needed; for the next release take the listing immediately before updating (reference SHA-256 `8bb0a913ca5fd3ef2fcc842b2d1643a30aeaa1566a150b8c41c9666852caa851`).
- **Decision**: ACCEPTED — equivalent evidence, gap disclosed; guidance applies to S-05's release (0.1.8).

### F6 — Stale comment in AppShell

- **Severity**: OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Pattern Consistency
- **Location**: src/AppShell.tsx:78
- **Detail**: "…keeps the form mounted, so its fuel, date and result survive" — the result now lives in `useFillUpEditing`.
- **Fix**: Reword the comment.
- **Decision**: FIXED — "…so its fuel and date survive; the last-action notice lives in useFillUpEditing."

## Triage summary (2026-11-21)

- Fixed: F1 (Fix A), F2, F3, F4, F6
- Accepted: F5
- Shipping: fixes land on `main` without a release of their own; S-05 rebases and its 0.1.8 ships them. Manual check of Fix A is folded into S-05's Phase 4 real-data run.
