import { useState, type FormEvent } from "react";
import { errorMessage } from "../vault/api";
import {
  FUEL_LABELS,
  U32_MAX,
  addFillUp,
  type Fuel,
  type FillUpRejection,
  type FillUpResult,
  type StartPointView,
} from "./api";
import { FillUpResultPanel } from "./FillUpResultPanel";
import { formatDate, formatKm, isIsoDate, parseDecimal2, parseKm, todayIso } from "./format";

type Props = {
  startPoint: StartPointView;
  /** A fill-up was saved; the shell refreshes the fuel cards. */
  onSaved: () => void;
  /** The backend has no start point after all; the shell re-reads its state. */
  onNoStartPoint: () => void;
};

type Saved = { fuel: Fuel; date: string; result: FillUpResult };

function rejectionMessage(reason: FillUpRejection, start: StartPointView): string {
  switch (reason.kind) {
    case "invalidDate":
      return "Nieprawidłowa data.";
    case "odometerOutOfRange":
      return "Przebieg musi mieścić się w zakresie 1–9 999 999 km.";
    case "litresOutOfRange":
      return "Litry muszą mieścić się w zakresie 0,01–1000,00 l.";
    case "amountOutOfRange":
      return "Kwota musi mieścić się w zakresie 0,01–100 000,00 zł.";
    case "dateBeforeStart":
      return `Data jest wcześniejsza niż data punktu startowego (${formatDate(start.date)}).`;
    case "odometerNotAfterStart":
      return `Przebieg musi być większy niż przebieg punktu startowego (${formatKm(start.odometerKm)}).`;
    case "sameFuelSameOdometer":
      return `Tankowanie tego paliwa przy tym przebiegu już istnieje (${formatDate(reason.date)}).`;
    case "orderConflict":
      return `Wpis koliduje z tankowaniem z ${formatDate(reason.date)} przy ${formatKm(reason.odometerKm)} (przebieg i data muszą rosnąć razem).`;
  }
}

/** New fill-up form; shows the computed result of the entry it just saved. */
export function FillUpForm({ startPoint, onSaved, onNoStartPoint }: Props) {
  const [fuel, setFuel] = useState<Fuel>("lpg");
  const [date, setDate] = useState(todayIso);
  const [odometer, setOdometer] = useState("");
  const [litres, setLitres] = useState("");
  const [amount, setAmount] = useState("");
  const [fullTank, setFullTank] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [saved, setSaved] = useState<Saved | null>(null);

  const odometerKm = parseKm(odometer);
  const litresCl = parseDecimal2(litres);
  const amountGrosze = parseDecimal2(amount);
  const canSubmit =
    !busy && isIsoDate(date) && odometerKm !== null && litresCl !== null && amountGrosze !== null;

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!canSubmit || odometerKm === null || litresCl === null || amountGrosze === null) return;
    setSaved(null);
    // Values above u32 would fail argument deserialization; they are out of range either way.
    if (odometerKm > U32_MAX) {
      setError(rejectionMessage({ kind: "odometerOutOfRange" }, startPoint));
      return;
    }
    if (litresCl > U32_MAX) {
      setError(rejectionMessage({ kind: "litresOutOfRange" }, startPoint));
      return;
    }
    setBusy(true);
    setError("");
    try {
      const result = await addFillUp({ fuel, date, odometerKm, litresCl, amountGrosze, fullTank });
      switch (result.kind) {
        case "saved":
          setSaved({ fuel, date, result: result.result });
          // Fuel and date stay: the next entry is often the other fuel on the same day.
          setOdometer("");
          setLitres("");
          setAmount("");
          setFullTank(true);
          onSaved();
          break;
        case "rejected":
          setError(rejectionMessage(result.reason, startPoint));
          break;
        case "noStartPoint":
          onNoStartPoint();
          return;
      }
    } catch (err) {
      setError(errorMessage(err));
    }
    setBusy(false);
  }

  return (
    <section className="fill-up">
      <h2>Dodaj tankowanie</h2>
      <form className="gate-form" onSubmit={handleSubmit}>
        <fieldset className="fuel-choice" disabled={busy}>
          <legend>Paliwo</legend>
          {(Object.keys(FUEL_LABELS) as Fuel[]).map((f) => (
            <label key={f}>
              <input type="radio" name="fuel" value={f} checked={fuel === f} onChange={() => setFuel(f)} />
              {FUEL_LABELS[f]}
            </label>
          ))}
        </fieldset>
        <label htmlFor="fill-up-date">Data</label>
        <input
          id="fill-up-date"
          type="date"
          value={date}
          readOnly={busy}
          onChange={(e) => setDate(e.currentTarget.value)}
        />
        <label htmlFor="fill-up-odometer">Przebieg (km)</label>
        <input
          id="fill-up-odometer"
          inputMode="numeric"
          autoComplete="off"
          value={odometer}
          readOnly={busy}
          onChange={(e) => setOdometer(e.currentTarget.value)}
        />
        {odometer.trim().length > 0 && odometerKm === null && (
          <p className="gate-hint">Podaj przebieg w pełnych kilometrach (same cyfry).</p>
        )}
        <label htmlFor="fill-up-litres">Litry</label>
        <input
          id="fill-up-litres"
          inputMode="decimal"
          autoComplete="off"
          value={litres}
          readOnly={busy}
          onChange={(e) => setLitres(e.currentTarget.value)}
        />
        {litres.trim().length > 0 && litresCl === null && (
          <p className="gate-hint">Podaj liczbę z najwyżej 2 miejscami po przecinku, np. 46,35.</p>
        )}
        <label htmlFor="fill-up-amount">Kwota (zł)</label>
        <input
          id="fill-up-amount"
          inputMode="decimal"
          autoComplete="off"
          value={amount}
          readOnly={busy}
          onChange={(e) => setAmount(e.currentTarget.value)}
        />
        {amount.trim().length > 0 && amountGrosze === null && (
          <p className="gate-hint">Podaj kwotę z najwyżej 2 miejscami po przecinku, np. 141,37.</p>
        )}
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={fullTank}
            disabled={busy}
            onChange={(e) => setFullTank(e.currentTarget.checked)}
          />
          Tankowanie do pełna
        </label>
        {error && (
          <p className="gate-error" role="alert">
            {error}
          </p>
        )}
        <button type="submit" disabled={!canSubmit}>
          {busy ? "Zapisywanie…" : "Zapisz tankowanie"}
        </button>
      </form>
      {saved && <FillUpResultPanel fuel={saved.fuel} date={saved.date} result={saved.result} />}
    </section>
  );
}
