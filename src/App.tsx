import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { AppShell } from "./AppShell";
import { VaultGate } from "./vault/VaultGate";
import "./App.css";

function App() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [updateState, setUpdateState] = useState<"idle" | "installing" | "failed">("idle");

  useEffect(() => {
    // Background check; offline or any other failure is ignored so the app stays fully usable without network.
    check({ timeout: 5000 })
      .then(setUpdate)
      .catch(() => {});
  }, []);

  async function installUpdate() {
    if (!update) return;
    setUpdateState("installing");
    try {
      // On Windows the NSIS installer closes the app itself.
      await update.downloadAndInstall();
    } catch {
      setUpdateState("failed");
    }
  }

  return (
    <main className="container">
      {/* Outside the gate so a locked or too-new vault can still be fixed by updating. */}
      {update && (
        <div className="update-banner">
          <span>
            {updateState === "failed"
              ? "Nie udało się pobrać aktualizacji."
              : `Dostępna wersja ${update.version}.`}
          </span>
          <button type="button" onClick={installUpdate} disabled={updateState === "installing"}>
            {updateState === "installing" ? "Pobieranie…" : updateState === "failed" ? "Spróbuj ponownie" : "Zainstaluj"}
          </button>
        </div>
      )}
      <VaultGate>
        <AppShell />
      </VaultGate>
    </main>
  );
}

export default App;
