---
change_id: release-runs-tests
title: Release runs tests
status: implementing
created: 2026-10-11
updated: 2026-10-11
archived_at: null
---

## Notes

### Phase 2 evidence

- 2026-10-11: red run 38127827929 from throwaway branch `release-gate-redcheck` (commit e28396c: MIN_PASSWORD_CHARS 8 -> 7, version 0.0.0) — conclusion failure at step "Rust tests" (23 passed; 1 failed: `password_shorter_than_8_chars_rejected`); `npm ci` and `tauri-action` skipped; releases and remote tags unchanged (app-v0.1.0…app-v0.1.3).
