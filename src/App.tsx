import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { check, type Update } from "@tauri-apps/plugin-updater";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
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

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container">
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
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>
    </main>
  );
}

export default App;
