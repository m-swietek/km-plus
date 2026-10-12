import { useState, type FormEvent } from "react";
import { errorMessage } from "../vault/api";
import { U32_MAX, setStartPoint, type StartPointRejection } from "./api";
import { formatDate, formatKm, isIsoDate, parseKm } from "./format";

type Props = {
  /** Saved, or already set elsewhere: the shell re-reads the fuel overview. */
  onDone: () => void;
};

const REJECTION_MESSAGES: Record<StartPointRejection, string> = {
  invalidDate: "Nieprawidłowa data.",
  odometerOutOfRange: "Przebieg musi mieścić się w zakresie 0–9 999 999 km.",
};

/** One-time start point: date and odometer when both tanks were full, confirmed before saving. */
export function StartPointScreen({ onDone }: Props) {
  const [date, setDate] = useState("");
  const [odometer, setOdometer] = useState("");
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);
  const [rejection, setRejection] = useState<StartPointRejection | null>(null);
  const [error, setError] = useState("");

  const odometerKm = parseKm(odometer);
  const canContinue = isIsoDate(date) && odometerKm !== null;

  function handleContinue(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!canContinue) return;
    setRejection(null);
    setError("");
    setConfirming(true);
  }

  async function save() {
    if (odometerKm === null || busy) return;
    if (odometerKm > U32_MAX) {
      // Would fail argument deserialization; it is out of range either way.
      setRejection("odometerOutOfRange");
      setConfirming(false);
      return;
    }
    setBusy(true);
    setError("");
    try {
      const result = await setStartPoint(date, odometerKm);
      switch (result.kind) {
        case "ok":
        case "alreadySet":
          onDone();
          return;
        case "invalid":
          setRejection(result.reason);
          setConfirming(false);
          break;
      }
    } catch (err) {
      setError(errorMessage(err));
    }
    setBusy(false);
  }

  if (confirming && odometerKm !== null) {
    return (
      <section className="gate">
        <h1>Punkt startowy</h1>
        <p>
          Punkt startowy: <strong>{formatDate(date)}</strong>, <strong>{formatKm(odometerKm)}</strong>. Czy dane
          są poprawne?
        </p>
        {error && (
          <p className="gate-error" role="alert">
            {error}
          </p>
        )}
        <div className="gate-actions">
          <button type="button" onClick={save} disabled={busy}>
            {busy ? "Zapisywanie…" : "Zapisz"}
          </button>
          <button type="button" onClick={() => setConfirming(false)} disabled={busy}>
            Popraw
          </button>
        </div>
      </section>
    );
  }

  return (
    <section className="gate">
      <h1>Punkt startowy</h1>
      <p>
        Podaj datę i przebieg chwili, w której oba baki — gaz i benzyna — były zatankowane do pełna. Punktu
        startowego nie można później zmienić.
      </p>
      <form className="gate-form" onSubmit={handleContinue}>
        <label htmlFor="start-date">Data</label>
        <input
          id="start-date"
          type="date"
          autoFocus
          value={date}
          onChange={(e) => setDate(e.currentTarget.value)}
        />
        {rejection === "invalidDate" && (
          <p className="gate-error" role="alert">
            {REJECTION_MESSAGES.invalidDate}
          </p>
        )}
        <label htmlFor="start-odometer">Przebieg (km)</label>
        <input
          id="start-odometer"
          inputMode="numeric"
          autoComplete="off"
          value={odometer}
          onChange={(e) => setOdometer(e.currentTarget.value)}
        />
        {odometer.trim().length > 0 && odometerKm === null && (
          <p className="gate-hint">Podaj przebieg w pełnych kilometrach (same cyfry).</p>
        )}
        {rejection === "odometerOutOfRange" && (
          <p className="gate-error" role="alert">
            {REJECTION_MESSAGES.odometerOutOfRange}
          </p>
        )}
        <button type="submit" disabled={!canContinue}>
          Dalej
        </button>
      </form>
    </section>
  );
}
