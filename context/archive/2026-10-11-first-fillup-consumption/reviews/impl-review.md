<!-- IMPL-REVIEW-REPORT -->
# Implementation Review: Pierwsze tankowanie daje spalanie (S-02)

- **Plan**: context/changes/first-fillup-consumption/plan.md
- **Scope**: Full plan
- **Reviewed phases**: 1, 2, 3, 4
- **Date**: 2026-10-18
- **Verdict**: NEEDS ATTENTION
- **Findings**: 0 critical, 1 warning, 5 observations

## Verdicts

| Dimension | Verdict |
|-----------|---------|
| Plan Adherence | WARNING |
| Scope Discipline | WARNING |
| Safety & Quality | WARNING |
| Architecture | PASS |
| Pattern Consistency | WARNING |
| Success Criteria | WARNING |

Automated criteria re-run on 2026-10-18: `cargo test` (59 passed, incl. `oracle_sample_matches_hand_computed_results`), `cargo check`, `grep` for the three commands, `npm run build`, `npm audit` (0 vulnerabilities), release run 38215438069 success with step "Rust tests: success", release `app-v0.1.4` with installer, `.sig`, `latest.json` (already published). All named unit tests from Testing Strategy exist; consumption rule, validation, migration sequencing and `commit()` rollback match the plan.

## Findings

### F1 — Phase 4 evidence trail never recorded

- **Severity**: ⚠️ WARNING
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Plan Adherence / Success Criteria
- **Location**: context/changes/first-fillup-consumption/change.md (## Notes), context/deployment/deploy-plan.md:112 (## Przebieg wykonania)
- **Detail**: Phase 4 §2 (plan.md:301-307) requires appending the run number with green "Rust tests", SHA-256 of `kmplus.vault` before the update and after unlock-without-save, and the `pre-v2-*` filename with its SHA-256. Neither file was touched; 45a1fae only flips Progress rows. Manual items 4.3–4.5 are `[x]` on the user's verbal confirmation with no recorded values — the migration proof the plan wanted as durable evidence does not exist in the repo.
- **Fix**: Append the evidence: run 38215438069 (Rust tests green), the `kmplus.vault.pre-v2-*` filename and its SHA-256 (computable now from `%APPDATA%\io.github.m-swietek.kmplus\`), and the pre-update hash the user noted.
- **Decision**: FIXED — during triage the data dir held only the v1 vault (no `pre-v2-*`, no `.bak`), so the first 4.5 confirmation had left no trace. User redid the check on 2026-10-18: `kmplus.vault.pre-v2-1792319089` SHA-256 = v1 baseline `43c94592…0143e4`. Evidence recorded in change.md (## Dowód wydania i migracji) and deploy-plan.md (## Przebieg wykonania). Restart-persistence part of 4.5 confirmed by user on 2026-10-18 (result visible after restart).

### F2 — Pre-migration flag cleared even when no copy was written

- **Severity**: 💡 OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Plan Adherence
- **Location**: src-tauri/src/vault/mod.rs:140
- **Detail**: Plan (plan.md:67): "znacznik czyszczony tylko po udanej kopii". `self.pre_migration_copy_due = false;` sits outside `if let Some(previous)`, so if the main file is missing at save time the flag clears without a copy. Only reachable if `kmplus.vault` is deleted externally between unlock and first save (`restore_backup` always rewrites main first), and then there are no v1 bytes on main to copy. No test covers it.
- **Fix**: Move the flag reset inside the `if let Some(previous)` block, after `write_synced`.
- **Decision**: FIXED (differently) — the proposed fix would make the next save copy v2 bytes under the `pre-v2-*` name, so current behaviour is correct; the plan's wording just missed the no-main-file case. Added an explanatory comment at mod.rs:126-129 instead.

### F3 — Pre-v2 copy written directly to its final name

- **Severity**: 💡 OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Safety & Quality
- **Location**: src-tauri/src/vault/mod.rs:138
- **Detail**: Every other durable write in `save()` goes `.tmp` → `rename`; the pre-migration copy is written straight to `kmplus.vault.pre-v2-<s>`. A crash or full disk mid-write leaves a truncated file under the trusted recovery name. No data loss (main untouched, flag stays set, next save writes a complete `-1` copy), but the leftover misleads manual recovery.
- **Fix**: Write to `<copy>.tmp`, then `fs::rename` to the final name, matching the existing pattern.
- **Decision**: FIXED — copy now written to `kmplus.vault.pre.tmp` (`PRE_MIGRATION_TMP_FILE`) and renamed; module doc and contract-surfaces.md updated; `cargo test` 59 passed.

### F4 — Blocking file I/O inside async fuel commands

- **Severity**: 💡 OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Pattern Consistency
- **Location**: src-tauri/src/fuel/commands.rs:178-212
- **Detail**: `set_start_point` / `add_fill_up` do synchronous read + fsync + rename while holding the std `Mutex` on the async runtime thread; sibling vault commands deliberately use `spawn_blocking` (vault/commands.rs:76, 125). Negligible at current data sizes.
- **Fix**: Make the fuel commands plain sync `fn` (Tauri runs those off the main thread), or wrap the body in `spawn_blocking`.
- **Decision**: SKIPPED

### F5 — Two JSON shapes for rejection reasons

- **Severity**: 💡 OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Pattern Consistency
- **Location**: src-tauri/src/fuel/commands.rs:57-62 vs 100-111
- **Detail**: `SetStartPointResult.invalid.reason` is a bare string (`"invalidDate"`), `AddFillUpResult.rejected.reason` is a tagged object (`{"kind":"invalidDate"}`). Both match the plan's contract, are documented in contract-surfaces.md, typed in api.ts and pinned by shape tests — the inconsistency originates in the plan itself.
- **Fix**: Leave as is (contract is pinned); if consistency is wanted, change it now before more consumers exist (Rust view, TS type, doc, test).
- **Decision**: FIXED — `SetStartPointResult.invalid.reason` is now `{kind}`-tagged like `FillUpRejection` (deliberate change from plan.md:186 contract); updated StartPointRejectionView serde tag, shape tests, `src/fuel/api.ts`, `StartPointScreen.tsx`, contract-surfaces.md. `cargo test` 59 passed, `npm run build` clean.

### F6 — Small unplanned UI behaviours

- **Severity**: 💡 OBSERVATION
- **Impact**: 🏃 LOW — quick decision; fix is obvious and narrowly scoped
- **Dimension**: Scope Discipline
- **Location**: src/fuel/FillUpResultPanel.tsx:14-16, src/fuel/FillUpForm.tsx:88
- **Detail**: Extra heading "Zapisano: <Paliwo>, DD.MM.RRRR" above the result; the "do pełna" checkbox resets to checked after save (plan only said fuel and date stay). Also benign defensive extras: `U32_MAX` guard before `invoke`, extra command-level tests. None crosses "What We're NOT Doing".
- **Fix**: Accept as-is (benign, verified manually in 3.5).
- **Decision**: ACCEPTED — benign extras, kept as-is (user decision 2026-10-18).
