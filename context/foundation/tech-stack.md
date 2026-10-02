---
starter_id: tauri
package_manager: cargo
project_name: km-plus
hints:
  language_family: rust
  team_size: solo
  deployment_target: self-host
  ci_provider: github-actions
  ci_default_flow: manual-promotion
  bootstrapper_confidence: verified
  path_taken: standard
  quality_override: false
  self_check_answers: null
  has_auth: true
  has_payments: false
  has_realtime: false
  has_ai: false
  has_background_jobs: false
---

## Why this stack

Jednoosobowy projekt desktopowy budowany po godzinach w 6 tygodni, z twardym wymaganiem pracy w pełni offline i bez wysyłki danych na zewnątrz. Tauri jest wetowanym domyślnym starterem dla pary (desktop, Rust) i przechodzi wszystkie cztery bramki agent-friendly: jawne typy po obu stronach (Rust + TypeScript), silne konwencje układu projektu, obecność w danych treningowych i aktualna dokumentacja. Scaffolding jest sprawdzony end-to-end, więc start projektu nie zje budżetu czasowego. Backend w Ruście jest też właściwym miejscem na wymaganie z FR-001 i wymagań niefunkcjonalnych: lokalna baza szyfrowana kluczem wyprowadzonym z hasła użytkownika — warstwa, której starter nie niesie i którą trzeba dołożyć ręcznie zaraz po zescaffoldowaniu. Dystrybucja to self-host (własny build instalatora), zgodnie z domyślną wartością karty i z modelem jednego użytkownika na jednym komputerze. CI na GitHub Actions z ręczną promocją — wydanie instalatora pozostaje świadomą decyzją, nie efektem ubocznym merge'a. Flagi funkcji: auth włączony; płatności, realtime, AI i zadania w tle są wprost poza zakresem według sekcji Non-Goals.
