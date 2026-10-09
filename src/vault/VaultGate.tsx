import { useCallback, useEffect, useState, type ReactNode } from "react";
import {
  errorMessage,
  formatBackupDate,
  vaultRestoreBackup,
  vaultStatus,
  vaultUnlock,
  type UnlockResult,
} from "./api";
import { PasswordScreen } from "./PasswordScreen";
import { SetupScreen } from "./SetupScreen";

const DATA_DIR = "%APPDATA%\\io.github.m-swietek.kmplus";

type Screen =
  | { kind: "loading" }
  | { kind: "setup" }
  | { kind: "unlock"; error?: string }
  | { kind: "missingWithBackup"; backupSavedAt: number }
  // The password is kept only while the user decides; any other screen drops it.
  | { kind: "damagedBackupAvailable"; backupSavedAt: number; password: string }
  | { kind: "damagedNoBackup" }
  | { kind: "tooNew" }
  | { kind: "error"; message: string }
  | { kind: "unlocked" };

/** Shows the screen matching the vault state and renders `children` only once the vault is unlocked. */
export function VaultGate({ children }: { children: ReactNode }) {
  const [screen, setScreen] = useState<Screen>({ kind: "loading" });

  const loadStatus = useCallback(async () => {
    setScreen({ kind: "loading" });
    try {
      const status = await vaultStatus();
      switch (status.kind) {
        case "needsSetup":
          setScreen({ kind: "setup" });
          break;
        case "locked":
          setScreen({ kind: "unlock" });
          break;
        case "missingWithBackup":
          setScreen({ kind: "missingWithBackup", backupSavedAt: status.backupSavedAt });
          break;
        case "unlocked":
          setScreen({ kind: "unlocked" });
          break;
      }
    } catch (err) {
      setScreen({ kind: "error", message: errorMessage(err) });
    }
  }, []);

  useEffect(() => {
    void loadStatus();
  }, [loadStatus]);

  const showError = useCallback((message: string) => setScreen({ kind: "error", message }), []);

  const applyUnlockResult = useCallback((result: UnlockResult, password: string) => {
    switch (result.kind) {
      case "unlocked":
        setScreen({ kind: "unlocked" });
        break;
      case "wrongPassword":
        setScreen({ kind: "unlock", error: "Nieprawidłowe hasło" });
        break;
      case "damagedBackupAvailable":
        setScreen({ kind: "damagedBackupAvailable", backupSavedAt: result.backupSavedAt, password });
        break;
      case "damagedNoBackup":
        setScreen({ kind: "damagedNoBackup" });
        break;
      case "tooNew":
        setScreen({ kind: "tooNew" });
        break;
    }
  }, []);

  switch (screen.kind) {
    case "loading":
      return (
        <section className="gate">
          <p>Wczytywanie…</p>
        </section>
      );

    case "setup":
      return (
        <SetupScreen
          onUnlocked={() => setScreen({ kind: "unlocked" })}
          onAlreadyExists={loadStatus}
          onError={showError}
        />
      );

    case "unlock":
      return (
        <PasswordScreen
          title="Odblokuj kmPlus"
          submitLabel="Odblokuj"
          busyLabel="Odblokowywanie…"
          initialError={screen.error}
          submit={vaultUnlock}
          onResult={applyUnlockResult}
          onError={showError}
        />
      );

    case "missingWithBackup":
      return (
        <PasswordScreen
          title="Brak pliku danych"
          submitLabel="Przywróć kopię"
          busyLabel="Przywracanie…"
          submit={vaultRestoreBackup}
          onResult={applyUnlockResult}
          onError={showError}
        >
          <p>
            Nie znaleziono głównego pliku danych. Istnieje kopia zapasowa z{" "}
            <strong>{formatBackupDate(screen.backupSavedAt)}</strong>.
          </p>
          <p>Podaj hasło, aby przywrócić dane z kopii.</p>
        </PasswordScreen>
      );

    case "damagedBackupAvailable":
      return (
        <DamagedBackupScreen
          backupSavedAt={screen.backupSavedAt}
          password={screen.password}
          onResult={applyUnlockResult}
          onCancel={() => setScreen({ kind: "unlock" })}
          onError={showError}
        />
      );

    case "damagedNoBackup":
      return (
        <section className="gate">
          <h1>Plik danych jest uszkodzony</h1>
          <p>
            Pliku danych nie da się odczytać, a nie ma kopii zapasowej, z której można by go przywrócić.
            Aplikacja niczego nie zmieniła na dysku.
          </p>
          <p>
            Pliki danych znajdują się w katalogu: <code>{DATA_DIR}</code>
          </p>
        </section>
      );

    case "tooNew":
      return (
        <section className="gate">
          <h1>Dane z nowszej wersji</h1>
          <p>
            Dane zostały zapisane przez nowszą wersję kmPlus. Zaktualizuj aplikację, aby je otworzyć. Ta wersja
            ich nie otworzy ani nie zmieni.
          </p>
        </section>
      );

    case "error":
      return (
        <section className="gate">
          <h1>Wystąpił błąd</h1>
          <p className="gate-error" role="alert">
            {screen.message}
          </p>
          <button type="button" onClick={loadStatus}>
            Spróbuj ponownie
          </button>
        </section>
      );

    case "unlocked":
      return <>{children}</>;
  }
}

type DamagedBackupProps = {
  backupSavedAt: number;
  password: string;
  onResult: (result: UnlockResult, password: string) => void;
  onCancel: () => void;
  onError: (message: string) => void;
};

function DamagedBackupScreen({ backupSavedAt, password, onResult, onCancel, onError }: DamagedBackupProps) {
  const [busy, setBusy] = useState(false);

  async function restore() {
    setBusy(true);
    try {
      onResult(await vaultRestoreBackup(password), password);
    } catch (err) {
      onError(errorMessage(err));
    }
    // Only matters if the parent kept this screen mounted (same outcome again).
    setBusy(false);
  }

  return (
    <section className="gate">
      <h1>Plik danych jest uszkodzony</h1>
      <p>
        Główny plik danych jest uszkodzony. Ostatnia dobra kopia pochodzi z{" "}
        <strong>{formatBackupDate(backupSavedAt)}</strong>.
      </p>
      <p>Przywrócenie zastąpi uszkodzony plik tą kopią; uszkodzony plik zostanie zachowany pod inną nazwą.</p>
      <div className="gate-actions">
        <button type="button" onClick={restore} disabled={busy}>
          {busy ? "Przywracanie…" : "Przywróć kopię"}
        </button>
        <button type="button" onClick={onCancel} disabled={busy}>
          Anuluj
        </button>
      </div>
    </section>
  );
}
