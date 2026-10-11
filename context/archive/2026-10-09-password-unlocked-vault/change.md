---
change_id: password-unlocked-vault
title: Hasło odblokowuje zaszyfrowane dane (roadmapa S-01)
status: archived
created: 2026-10-09
updated: 2026-10-11
archived_at: 2026-10-11T09:05:13Z
---

## Notes

S-01 z @context/foundation/roadmap.md

### Phase 4 evidence

- 2026-10-09: release 0.1.2 — run 37989636644 success; draft `app-v0.1.2` (`km-plus_0.1.2_x64-setup.exe` 2 299 007 B, `.sig`, `latest.json`; signature in `latest.json` matches `.sig`). Published manually; public `latest.json` serves 0.1.2 (checked 2026-10-11).
- 2026-10-11: on installed 0.1.2, SHA-256 of `kmplus.vault` before the 0.1.2 → 0.1.3 update: `43C94592A5ADD3A3E4E4A912768E7DD194ABE70CD719D0CB5B32FB332A0143E4`
- 2026-10-11: release 0.1.3 — run 38126357536 success; draft `app-v0.1.3` (`km-plus_0.1.3_x64-setup.exe` 2 298 052 B, `.sig`, `latest.json`; signature matches). Published manually.
- 2026-10-11: installed 0.1.2 updated in place to 0.1.3 via the banner; SHA-256 of `kmplus.vault` after the update: `43C94592A5ADD3A3E4E4A912768E7DD194ABE70CD719D0CB5B32FB332A0143E4` (identical to before); the same password unlocked 0.1.3 (no setup screen).
