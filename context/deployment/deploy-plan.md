# Pierwsze wdrożenie kmPlus: GitHub Actions → GitHub Releases (draft) → Tauri updater

## Context

kmPlus to aplikacja desktopowa offline, więc „wdrożenie" oznacza: zbudowanie instalatora Windows w CI, opublikowanie go w GitHub Releases i umożliwienie zainstalowanym kopiom wykrycia nowej wersji. `infrastructure.md` wskazuje GitHub Actions + Releases + updater Tauri; `tech-stack.md` wymaga ręcznej promocji (`ci_default_flow: manual-promotion`).

Stan zastany (sprawdzony tylko do odczytu):

- Repo `m-swietek/km-plus` jest **już publiczne**; lokalny `main` jest 2 commity przed `origin/main` (niewypchnięte).
- Kod to nietknięty scaffold; `tauri.conf.json` ma `identifier: com.administrator.km-plus` i `bundle.targets: "all"`.
- Brak `.github/`, brak pluginu updatera, brak klucza `~/.tauri/km-plus.key`.
- `gh` nie jest zainstalowany (jest `winget`). `cargo` 1.99.0 istnieje w `~\.cargo\bin`, ale **nie ma go w PATH** — każde polecenie cargo/tauri trzeba poprzedzić `$env:PATH = "$HOME\.cargo\bin;$env:PATH"`.

Decyzje podjęte w tej sesji:

- Identyfikator: `io.github.m-swietek.kmplus` (zamrożony od pierwszego wydania).
- Trigger: wyłącznie ręczny `workflow_dispatch`; wydanie zawsze jako draft.
- Updater: plugin + sprawdzanie w tle już w 0.1.0.
- `gh` CLI instalujemy; logowanie i sekrety wykonuje człowiek.

Do potwierdzenia przy akceptacji planu: `productName` zostaje `km-plus` (nazwa pliku exe i katalogu instalacji). Jeśli ma być inna, trzeba to zmienić teraz, przed 0.1.0.

## Kroki

### 0. Zapis planu
Zapisać zatwierdzony plan jako `context/deployment/deploy-plan.md` (nowy katalog).

### 1. Tożsamość aplikacji — `src-tauri/tauri.conf.json`
- `identifier` → `io.github.m-swietek.kmplus`
- `bundle.targets` → `["nsis"]` (jeden typ instalatora; ryzyko „MSI + NSIS" z rejestru)

### 2. Plugin updatera
- `npm run tauri add updater` (z cargo w PATH). Sprawdzić, że wykonał wszystkie trzy zmiany wymagane przez AGENTS.md, i uzupełnić ręcznie, jeśli którejś brakuje:
  - `src-tauri/Cargo.toml` — `tauri-plugin-updater`
  - `src-tauri/src/lib.rs` — rejestracja pluginu w `run()`
  - `src-tauri/capabilities/default.json` — `updater:default`
- `package.json` — `@tauri-apps/plugin-updater`

### 3. BRAMKA RĘCZNA — klucz updatera (wykonuje użytkownik)
Agent nie dotyka klucza prywatnego ani hasła.
1. We własnym terminalu: `npm run tauri signer generate -- -w $HOME\.tauri\km-plus.key` (hasło podawane interaktywnie).
2. Klucz prywatny i hasło od razu do menedżera haseł — utrata = zainstalowane kopie nigdy się nie zaktualizują.
3. Potwierdzić agentowi, że gotowe. Agent czyta wyłącznie `~/.tauri/km-plus.key.pub`.

### 4. Konfiguracja updatera — `src-tauri/tauri.conf.json`
- `bundle.createUpdaterArtifacts: true`
- `plugins.updater.pubkey` — treść pliku `.key.pub` (string, nie ścieżka)
- `plugins.updater.endpoints: ["https://github.com/m-swietek/km-plus/releases/latest/download/latest.json"]`

### 5. Sprawdzanie aktualizacji w tle — `src/App.tsx`
Minimalny kod, bez nowych pluginów poza updaterem:
- po zamontowaniu: `check({ timeout: 5000 })` z `@tauri-apps/plugin-updater` w `try/catch`; każdy błąd i brak sieci są po cichu ignorowane, nic nie blokuje renderowania;
- jeśli jest nowa wersja: nieblokujący baner „Dostępna wersja X" z przyciskiem „Zainstaluj"; dopiero kliknięcie wywołuje `downloadAndInstall()` (na Windows instalator NSIS sam zamyka aplikację — plugin `process` niepotrzebny).
- Żądanie nie niesie żadnych danych użytkownika (zgodne z ograniczeniem PRD przyjętym w `infrastructure.md`).

### 6. Workflow — `.github/workflows/release.yml` (nowy)
- `on: workflow_dispatch` (jedyny trigger), `permissions: contents: write`, `runs-on: windows-latest`
- kroki: checkout → setup-node (cache npm) → toolchain Rust stable + cache `src-tauri` → `npm ci` → `tauri-apps/tauri-action@v1`
- `with`: `tagName: app-v__VERSION__`, `releaseName: kmPlus v__VERSION__`, `releaseDraft: true`, `prerelease: false`
- `env`: `GITHUB_TOKEN`, `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` z sekretów
- Przed zapisem zweryfikować aktualne wersje akcji i nazwy wejść w https://v2.tauri.app/distribute/pipelines/github/ i README `tauri-apps/tauri-action`.

### 7. README.md
Krótka sekcja instalacji: ostrzeżenie SmartScreen („Więcej informacji → Uruchom mimo to"), instalacja wymaga internetu tylko gdy brakuje WebView2, oraz że zaznaczenie „usuń dane aplikacji" przy deinstalacji kasuje jedyną kopię historii.

### 8. Weryfikacja lokalna (przed commitem)
- `npm install`, `npm run build` (tsc strict + vite)
- `cargo check` w `src-tauri/`
- `npm audit`, `cargo audit`
- Pełnego `tauri build` lokalnie nie uruchamiamy — wymagałby klucza prywatnego w środowisku agenta.

### 9. Commit i push
- Przejrzeć diff i 2 niewypchnięte commity pod kątem sekretów (repo jest publiczne; `.claude/`, `CLAUDE.md`, `.10x-cli.json` są w `.gitignore`).
- Commit bezpośrednio na `main` (workflow_dispatch wymaga pliku workflow na gałęzi domyślnej; cała historia projektu jest na `main`), potem `git push origin main`.

### 10. BRAMKA RĘCZNA — gh i sekrety
1. Agent: `winget install --id GitHub.cli`.
2. Użytkownik: `gh auth login`.
3. Użytkownik ustawia sekrety (wartości nie trafiają do rozmowy):
   - `gh secret set TAURI_SIGNING_PRIVATE_KEY < $HOME\.tauri\km-plus.key` (w Git Bash; lub wklejenie w panelu Settings → Secrets → Actions)
   - `gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
4. Agent sprawdza tylko nazwy: `gh secret list`.

### 11. Build wydania
- Po wyraźnym „start" od użytkownika: `gh workflow run release.yml --ref main`, następnie `gh run watch` / `gh run view <id> --log` przy błędzie.
- Jeśli „Resource not accessible by integration": użytkownik włącza „Read and write permissions" w Settings → Actions → General.

### 12. Weryfikacja draftu i publikacja
- Agent: `gh release view app-v0.1.0` — draft zawiera instalator `*-setup.exe`, plik `.sig` i `latest.json`; `gh release download` do katalogu tymczasowego.
- Użytkownik: instaluje, uruchamia, sprawdza działanie bez sieci (start bez opóźnienia i bez błędu), sprawdza że dane trafiają do `%APPDATA%\io.github.m-swietek.kmplus`.
- **Użytkownik publikuje draft ręcznie** w panelu GitHub (agent tego nie robi).
- Agent po publikacji: `https://github.com/m-swietek/km-plus/releases/latest/download/latest.json` zwraca 200 z wersją 0.1.0.

### 13. Test ścieżki aktualizacji (0.1.0 → 0.1.1)
Jedyny sposób na realne sprawdzenie updatera. Podbić wersję do 0.1.1 w `tauri.conf.json`, `package.json`, `Cargo.toml`; powtórzyć kroki 9, 11, 12; po publikacji zainstalowana 0.1.0 pokazuje baner i aktualizuje się w miejscu (jedna kopia, nie dwie).

## Granice
- Tylko człowiek: generowanie i przechowywanie klucza, ustawianie sekretów, publikacja draftu, usuwanie wydań/tagów.
- Poza zakresem: podpisywanie kodu, winget, kopia bazy przed migracją (baza jeszcze nie istnieje — do dodania razem z warstwą persystencji), zmiana zachowania deinstalatora.

## Wycofanie
Cofnięcie wydania do draftu lub jego usunięcie (ręcznie) zatrzymuje pobrania i oferty aktualizacji; kopii już zaktualizowanych nie cofa — wymaga wydania wyższej wersji ze starym kodem.

## Przebieg wykonania

- 2026-10-02: kroki 0–10 wykonane; commit `5f678f4` na `main`. `gh` był już zainstalowany (`C:\Program Files\GitHub CLI\`, poza PATH). `tauri add updater` dodał uprawnienie w nowym `capabilities/desktop.json` zamiast `default.json`.
- 2026-10-02, run 37070244284: porażka przy podpisie updatera — `incorrect updater private key password`; sekret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` był pusty. Poprawiony ręcznie w panelu.
- 2026-10-03, run 37141751826: sukces. Draft `app-v0.1.0` z `km-plus_0.1.0_x64-setup.exe` (2 257 905 B, SHA-256 `A74DEFFB…E300D9`, Authenticode: NotSigned), `.sig` i `latest.json`; podpis w `latest.json` zgadza się z `.sig`. URL w `latest.json` wskazuje na API (`api.github.com/.../releases/assets/<id>`), nie `browser_download_url` — do potwierdzenia w teście 0.1.0 → 0.1.1.
- 2026-10-03: 0.1.0 opublikowane ręcznie (18:00 UTC). Anonimowo: `releases/latest/download/latest.json` → 200, wersja 0.1.0; URL instalatora z API z nagłówkiem `Accept: application/octet-stream` zwraca plik o tym samym SHA-256. Wersja podbita do 0.1.1 na potrzeby testu aktualizacji.

## Pliki
- zmieniane: `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`, `package.json`, `package-lock.json`, `src/App.tsx`, `src/App.css`, `README.md`
- nowe: `.github/workflows/release.yml`, `context/deployment/deploy-plan.md`
