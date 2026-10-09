import { useState, type FormEvent } from "react";
import { MIN_PASSWORD_LENGTH, errorMessage, passwordLength, vaultSetup } from "./api";

type Props = {
  onUnlocked: () => void;
  /** The vault appeared in the meantime; the gate re-queries its status. */
  onAlreadyExists: () => void;
  onError: (message: string) => void;
};

export function SetupScreen({ onUnlocked, onAlreadyExists, onError }: Props) {
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const longEnough = passwordLength(password) >= MIN_PASSWORD_LENGTH;
  const matches = password === confirmation;
  const canSubmit = longEnough && matches && !busy;

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!canSubmit) return;
    setBusy(true);
    setError("");
    try {
      const result = await vaultSetup(password);
      switch (result.kind) {
        case "ok":
          clear();
          onUnlocked();
          return;
        case "passwordTooShort":
          setError(`Hasło musi mieć co najmniej ${MIN_PASSWORD_LENGTH} znaków.`);
          setBusy(false);
          return;
        case "alreadyExists":
          clear();
          onAlreadyExists();
          return;
      }
    } catch (err) {
      clear();
      onError(errorMessage(err));
    }
  }

  function clear() {
    setPassword("");
    setConfirmation("");
  }

  return (
    <section className="gate">
      <h1>Ustaw hasło</h1>
      <p>Hasło szyfruje wszystkie dane kmPlus i będzie potrzebne przy każdym uruchomieniu aplikacji.</p>
      <p className="gate-warning" role="note">
        Hasła nie da się odzyskać. Jego utrata oznacza utratę wszystkich danych. Zapisz je w menedżerze haseł.
      </p>
      <form className="gate-form" onSubmit={handleSubmit}>
        <label htmlFor="setup-password">Hasło (co najmniej {MIN_PASSWORD_LENGTH} znaków)</label>
        <input
          id="setup-password"
          type="password"
          autoFocus
          autoComplete="new-password"
          value={password}
          readOnly={busy}
          onChange={(e) => setPassword(e.currentTarget.value)}
        />
        <label htmlFor="setup-confirmation">Powtórz hasło</label>
        <input
          id="setup-confirmation"
          type="password"
          autoComplete="new-password"
          value={confirmation}
          readOnly={busy}
          onChange={(e) => setConfirmation(e.currentTarget.value)}
        />
        {password.length > 0 && !longEnough && (
          <p className="gate-hint">Hasło musi mieć co najmniej {MIN_PASSWORD_LENGTH} znaków.</p>
        )}
        {confirmation.length > 0 && !matches && <p className="gate-hint">Hasła nie są zgodne.</p>}
        {error && (
          <p className="gate-error" role="alert">
            {error}
          </p>
        )}
        <button type="submit" disabled={!canSubmit}>
          {busy ? "Zakładanie…" : "Ustaw hasło"}
        </button>
      </form>
    </section>
  );
}
