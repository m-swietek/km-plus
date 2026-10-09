import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { errorMessage, type UnlockResult } from "./api";

type Props = {
  title: string;
  children?: ReactNode;
  submitLabel: string;
  busyLabel: string;
  initialError?: string;
  submit: (password: string) => Promise<UnlockResult>;
  /** Every outcome except `wrongPassword`, which this screen handles itself. */
  onResult: (result: UnlockResult, password: string) => void;
  onError: (message: string) => void;
};

/** Single password field used for unlocking and for restoring a missing main file from its backup. */
export function PasswordScreen({
  title,
  children,
  submitLabel,
  busyLabel,
  initialError,
  submit,
  onResult,
  onError,
}: Props) {
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(initialError ?? "");
  const inputRef = useRef<HTMLInputElement>(null);

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (busy || password.length === 0) return;
    setBusy(true);
    setError("");
    const entered = password;
    try {
      const result = await submit(entered);
      if (result.kind === "wrongPassword") {
        setPassword("");
        setError("Nieprawidłowe hasło");
        setBusy(false);
        inputRef.current?.focus();
        return;
      }
      setPassword("");
      onResult(result, entered);
    } catch (err) {
      setPassword("");
      onError(errorMessage(err));
    }
  }

  return (
    <section className="gate">
      <h1>{title}</h1>
      {children}
      <form className="gate-form" onSubmit={handleSubmit}>
        <label htmlFor="vault-password">Hasło</label>
        <input
          id="vault-password"
          ref={inputRef}
          type="password"
          autoFocus
          autoComplete="current-password"
          value={password}
          readOnly={busy}
          onChange={(e) => setPassword(e.currentTarget.value)}
        />
        {error && (
          <p className="gate-error" role="alert">
            {error}
          </p>
        )}
        <button type="submit" disabled={busy || password.length === 0}>
          {busy ? busyLabel : submitLabel}
        </button>
      </form>
    </section>
  );
}
