---
project: km-plus
researched_at: 2026-10-02
recommended_platform: GitHub Actions + GitHub Releases + Tauri updater
runner_up: CrabNebula Cloud
context_type: mvp
tech_stack:
  language: Rust + TypeScript
  framework: Tauri 2.12 + React 19
  runtime: native Windows desktop app (no server runtime)
---

## Recommendation

**Build on GitHub Actions, distribute through GitHub Releases, update with the Tauri updater plugin.**

kmPlus is an offline desktop app, so there is nothing to host: "infrastructure" here means how the Windows installer is built, where it is downloaded from, and how installed copies learn about new versions. The usual web-hosting candidates (Cloudflare, Vercel, Netlify, Fly.io, Railway, Render) were dropped up front because none runs a native desktop binary and the PRD forbids a hosted backend. Among distribution options, GitHub Releases is the only one that satisfies all five constraints from the interview at once: Windows only, zero cost, publicly downloadable, in-app auto-update, and no prior tooling experience (it has the official Tauri guide and action). It also matches `tech-stack.md` (`ci_provider: github-actions`, `ci_default_flow: manual-promotion`).

Two conditions come with it: the repository must be made public, and the installer ships unsigned.

## Platform Comparison

Scored against the five agent-friendly criteria, reinterpreted for distribution (deploy = publish a release; rollback = withdraw one).

| Option | CLI-first | Managed | Agent-readable docs | Stable deploy API | MCP / Integration | Total |
|---|---|---|---|---|---|---|
| GitHub Actions + Releases + updater | Pass | Pass | Pass | Pass | Pass | 5 Pass |
| CrabNebula Cloud | Pass | Pass | Partial | Pass | Partial | 3 Pass, 2 Partial |
| Local build + manual upload to Releases | Pass | Partial | Pass | Partial | Partial | 2 Pass, 3 Partial |
| winget (community repository) | Pass | Partial | Pass | Partial | Partial | not a host — add-on only |
| Self-hosted static files (Pages / R2) | Partial | Partial | Pass | Partial | Partial | dropped |
| Microsoft Store | Partial | Pass | Partial | Partial | Fail | dropped — cost |

- **GitHub Actions + Releases + updater** — `tauri-apps/tauri-action@v1` builds on a `windows-latest` runner, creates the release, and uploads the installer, its `.sig`, and `latest.json` for the updater (`uploadUpdaterJson` defaults to `true`). Standard GitHub-hosted runners are free for public repositories. GitHub has an official MCP server, and the Tauri docs are markdown on GitHub. Requires a public repo for anonymous downloads.
- **CrabNebula Cloud** — a CDN and update server built for Tauri, with a CLI and the `crabnebula-dev/cloud-release` action. Free for everyone, with billing only for unusually large traffic (checked 2026-10-02; the terms of that free offer could change). Works with a private repo. Costs a second vendor account and an API key secret; no MCP server was found; its docs format was not verified.
- **Local build + manual upload** — `npm run tauri build` on the developer's PC, then upload by hand. No CI to learn, but `latest.json` has to be assembled manually from the `.sig` output, the build is not reproducible from a clean machine, and the `gh` CLI is not installed locally.
- **winget** — a manifest in `microsoft/winget-pkgs` that points at the GitHub release installer, submitted with `wingetcreate` and accepted through an automated-plus-review pull request. It adds discoverability on top of a host; it does not replace one. Worth revisiting after a first release.
- **Self-hosted static files** — works with a private repo, but adds a storage bucket or Pages site, upload credentials, and a hand-maintained manifest for no benefit over Releases.
- **Microsoft Store** — individual developer registration is free (global since September 2025), but Tauri produces EXE/MSI, not MSIX, so the Store only links to an installer you host, and that installer must be signed with a certificate chaining to a Microsoft-trusted CA. Azure Artifact Signing ($9.99/month) accepts individual developers only in the USA and Canada; elsewhere an OV certificate runs roughly $150–300/year. Fails the zero-cost constraint.

### Shortlisted Platforms

#### 1. GitHub Actions + GitHub Releases + Tauri updater (Recommended)

Free, one vendor the project already uses, an official Tauri workflow that publishes the installer and the update manifest together, and a draft-release step that gives the manual promotion gate the tech stack asks for.

#### 2. CrabNebula Cloud

The best fit if the repository has to stay private: it hosts installers and serves update checks without exposing the source. It lost on the extra account, extra secret, and weaker agent integration.

#### 3. Local build + manual upload to GitHub Releases

The fallback if CI turns out to be a time sink in a six-week budget. Same download and update path for users, but every release is a manual, error-prone sequence and the signing key lives only on one machine.

## Anti-Bias Cross-Check: GitHub Actions + GitHub Releases + Tauri updater

### Devil's Advocate — Weaknesses

1. The installer is unsigned, so every downloader sees "Windows protected your PC", and each new version starts with no SmartScreen reputation. Antivirus false positives on NSIS installers are a known pattern.
2. The updater has its own signing key, separate from code signing, and it cannot be disabled. If the private key or its password is lost, already-installed copies can never be updated again.
3. The repository must be public, which exposes the full git history.
4. Auto-update is forward-only. A release that migrates the encrypted database cannot be undone by withdrawing the release, and the PRD treats data loss as a critical failure.
5. The update check is a network call from an app whose PRD promises fully offline operation. It carries no user data, but GitHub sees the IP address, and a check that hangs or errors offline would break that promise.

### Pre-Mortem — How This Could Fail

The first release ships with `bundle.targets: "all"`, so both an MSI and an NSIS installer are published. The update manifest prefers the MSI by default, so people who installed with the `.exe` are updated by a different installer type and end up with two copies. The bundle identifier is still the scaffold's `com.administrator.km-plus`; when it is later renamed to something presentable, the app looks for its data in a new folder and opens as if empty, and the fuel history appears lost. A later update changes the database schema with no backup step; one bug in that migration leaves the only copy of the history unreadable, with no recovery path by design. Finally, the updater key, stored only on one laptop, disappears with a Windows reinstall. Installed copies are now stranded on a version with a known bug, and the only fix is asking every user to uninstall and reinstall by hand — through an uninstaller that offers to delete their data.

### Unknown Unknowns

- The Tauri docs example creates the release as a draft. A draft is not served at `releases/latest/download/latest.json`, so nobody receives the update until the draft is published. This is the manual-promotion gate, not a bug.
- The app's data folder is derived from the bundle identifier. It must be final before the first public release.
- The NSIS uninstaller offers a "delete application data" checkbox. For this app that deletes the only copy of the history.
- The default installer downloads the WebView2 runtime if it is missing, so installing needs internet even though the app does not. Windows 11 ships WebView2; the offline installer mode adds about 127 MB.
- With `createUpdaterArtifacts: true` on Tauri 2, the update artifact is the installer itself plus a `.sig` file. The `"v1Compatible"` value is only for apps migrating from Tauri 1 and does not apply here.
- SignPath Foundation signs open-source projects for free, but only projects with an OSI licence that are already released and actively maintained, and the certificate names SignPath as publisher. It is a later upgrade, not a day-one option.

## Operational Story

- **Preview deploys**: none in the web sense. A release created as a draft is the preview: its installer can be downloaded from the draft page by the repo owner and tested before anyone else sees it. Pull requests do not produce installers.
- **Secrets**: `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` live in GitHub Actions repository secrets, readable only by workflows. A second copy of the key and password goes in the owner's password manager. `GITHUB_TOKEN` is issued per run. The public key is committed in `tauri.conf.json`. The key cannot be rotated without stranding installed copies.
- **Rollback**: withdrawing a release (delete it or revert it to draft) stops new downloads and update offers within seconds, but does not downgrade copies that already updated. Reverting those means publishing a higher version number containing the old code. Database migrations do not roll back.
- **Approval**: a human publishes the draft release, makes the repository public, and handles anything touching the updater key. An agent may bump the version, push the release trigger, watch the build, and download the draft installer for testing.
- **Logs**: `gh run list` and `gh run view <id> --log` (after installing the `gh` CLI), or the GitHub MCP server, for build logs. There are no runtime logs: the app runs on the user's machine and, by PRD decision, reports nothing back.

## Risk Register

| Risk | Source | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| Updater private key or password lost; installed copies can never update | Devil's advocate | M | H | Store key and password in the password manager and in GitHub secrets on the day they are generated; never keep the only copy on disk |
| Update with a faulty database migration destroys the only copy of the history | Devil's advocate / Pre-mortem | M | H | Copy the encrypted database file to a timestamped backup before any migration; test each release by updating from the previous version with real data before publishing the draft |
| Bundle identifier changed after first release orphans the data folder | Pre-mortem | M | H | Replace `com.administrator.km-plus` with a final identifier before the first public release, then treat it as frozen |
| MSI and NSIS both published; updater switches installer type and duplicates the install | Pre-mortem | H | M | Set `bundle.targets` to `["nsis"]` so only one installer type exists |
| Uninstall with "delete application data" ticked erases the history | Unknown unknowns | L | H | State it in the release notes and README; consider a warning in the app's own documentation |
| Update check hangs or errors with no network, breaking the offline guarantee | Devil's advocate | M | M | Run the check in the background with a short timeout and swallow failures; the app must be fully usable before and without it |
| SmartScreen warning or antivirus false positive deters downloaders | Devil's advocate | H | L | Document the "More info → Run anyway" step in the README; submit false positives to Microsoft; apply to SignPath Foundation once the project qualifies |
| Public repo exposes history | Devil's advocate | L | M | Before flipping visibility, scan the history for secrets; course material is already gitignored (`.claude/`, `CLAUDE.md`, `.10x-cli.json`) |
| Repository stays private; downloads and update checks return 404 | Research finding | M | H | Make the repo public before the first release, or switch to the runner-up (CrabNebula Cloud) |
| Draft release never published; users never see the update | Unknown unknowns | L | L | Publishing the draft is the last line of the release checklist |
| GitHub changes Actions pricing or runner images | Research finding | L | L | Public-repo usage on standard runners is free as of 2026-10-02; the local-build fallback needs no CI |

## Getting Started

Validated against the versions in this project: `tauri` 2.12.1, `@tauri-apps/cli` 2.12.1.

1. Finalise the app identity in `src-tauri/tauri.conf.json`: replace the `identifier` `com.administrator.km-plus` with the permanent one, and set `bundle.targets` to `["nsis"]`.
2. Add the updater plugin: `npm run tauri add updater`. Confirm it added `updater:default` to `src-tauri/capabilities/default.json`.
3. Generate the updater key: `npm run tauri signer generate -- -w ~/.tauri/km-plus.key`. Save the key and password in the password manager, then add them as the repository secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. In `tauri.conf.json`, set `bundle.createUpdaterArtifacts` to `true`, put the public key (the string itself, not a path) in `plugins.updater.pubkey`, and set `plugins.updater.endpoints` to `["https://github.com/m-swietek/km-plus/releases/latest/download/latest.json"]`.
5. Make the repository public, then add a workflow using `tauri-apps/tauri-action@v1` on `windows-latest` with `permissions: contents: write`, `tagName: app-v__VERSION__`, and `releaseDraft: true`. If the run fails with "Resource not accessible by integration", enable read and write workflow permissions in the repository's Actions settings.

Writing the workflow file itself is outside this document.

## Out of Scope

The following were not evaluated in this research:
- Docker image configuration
- CI/CD pipeline setup
- Production-scale architecture (multi-region, HA, DR)
- macOS and Linux builds
- Paid code-signing certificates beyond noting their cost

## Sources

- https://v2.tauri.app/plugin/updater/
- https://v2.tauri.app/distribute/pipelines/github/
- https://github.com/tauri-apps/tauri-action
- https://v2.tauri.app/distribute/sign/windows/
- https://v2.tauri.app/distribute/microsoft-store/
- https://v2.tauri.app/distribute/windows-installer/
- https://v2.tauri.app/distribute/crabnebula-cloud/
- https://docs.crabnebula.dev/cloud/org-management/billing/
- https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/
- https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options
- https://azure.microsoft.com/en-us/products/artifact-signing
- https://signpath.org/terms
- https://learn.microsoft.com/en-us/windows/package-manager/package/repository
