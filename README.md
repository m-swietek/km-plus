# kmPlus

Jednoosobowa aplikacja desktopowa offline dla samochodu z instalacją LPG: dziennik tankowań ze spalaniem liczonym osobno dla LPG i benzyny oraz rejestr serwisów z przypomnieniami. Tauri 2 + React + TypeScript.

## Instalacja (Windows)

Instalator `km-plus_<wersja>_x64-setup.exe` jest do pobrania z [najnowszego wydania](https://github.com/m-swietek/km-plus/releases/latest).

- **Ostrzeżenie SmartScreen.** Instalator nie jest podpisany certyfikatem, więc Windows pokaże „System Windows ochronił ten komputer". Wybierz **Więcej informacji → Uruchom mimo to**.
- **WebView2.** Jeśli na komputerze brakuje środowiska WebView2 (Windows 11 ma je wbudowane), instalator pobierze je z internetu. Sama aplikacja działa w pełni offline.
- **Aktualizacje.** Przy starcie aplikacja sprawdza w tle, czy jest nowa wersja. Zapytanie nie zawiera żadnych danych użytkownika, a jego niepowodzenie (np. brak sieci) jest ignorowane. Instalacja zaczyna się dopiero po kliknięciu „Zainstaluj".
- **Deinstalacja.** Deinstalator oferuje pole „usuń dane aplikacji". Zaznaczenie go kasuje jedyną kopię historii — bez możliwości odzyskania.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
