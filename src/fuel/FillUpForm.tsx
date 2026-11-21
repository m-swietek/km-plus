import { useState, type FormEvent } from "react";
import { errorMessage } from "../vault/api";
import {
  FUEL_LABELS,
  U32_MAX,
  addFillUp,
  updateFillUp,
  type Fuel,
  type FillUpRejection,
  type FillUpResult,
  type StartPointView,
} from "./api";
import { FillUpResultPanel } from "./FillUpResultPanel";
import {
  formatDate,
  formatDecimal2Input,
  formatKm,
  formatKmInput,
  isIsoDate,
  parseDecimal2,
  parseKm,
  todayIso,
} from "./format";
import { NOT_FOUND_MESSAGE, type Editing, type Notice, type Requests } from "./useFillUpEditing";

type Props = {
  startPoint: StartPointView;
  /** Data changed (or turned out stale); the shell refreshes the fuel cards and histories. */
  onSaved: () => void;
  /** The backend has no start point after all; the shell re-reads its state. */
  onNoStartPoint: () => void;
  /** Entry being corrected; `null` is the add mode. */
  editing: Editing | null;
  /** Status of the last action, shown under the form. */
  notice: Notice | null;
  onCancelEditing: () => void;
  onClearNotice: () => void;
  onAdded: (fuel: Fuel, date: string, result: FillUpResult) => void;
  onUpdated: (id: number, fuel: Fuel, date: string, result: FillUpResult) => void;
  /** The corrected entry no longer exists in the backend. */
  onGone: (id: number) => void;
  /** Writes in flight on the whole screen; the save goes through `track` and waits for the others. */
  requests: Requests;
};

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

/**
 * Fill-up form for adding a new entry or, while `editing` is set, correcting an existing one;
 * shows the status of the last action under it.
 */
export function FillUpForm({
  startPoint,
  onSaved,
  onNoStartPoint,
  editing,
  notice,
  onCancelEditing,
  onClearNotice,
  onAdded,
  onUpdated,
  onGone,
  requests,
}: Props) {
  const [fuel, setFuel] = useState<Fuel>("lpg");
  const [date, setDate] = useState(todayIso);
  const [odometer, setOdometer] = useState("");
  const [litres, setLitres] = useState("");
  const [amount, setAmount] = useState("");
  const [fullTank, setFullTank] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  /** The `editing` value the fields were last loaded from. */
  const [loaded, setLoaded] = useState<Editing | null>(null);

  /** Loads the fields from an entry, or returns to the add mode (fuel stays, date is today, rest empty). */
  function loadFields(next: Editing | null) {
    setLoaded(next);
    setError("");
    if (next) {
      setFuel(next.fuel);
      setDate(next.row.date);
      setOdometer(formatKmInput(next.row.odometerKm));
      setLitres(formatDecimal2Input(next.row.litresCl));
      setAmount(formatDecimal2Input(next.row.amountGrosze));
      setFullTank(next.row.fullTank);
    } else {
      setDate(todayIso());
      setOdometer("");
      setLitres("");
      setAmount("");
      setFullTank(true);
    }
  }

  // Entering, switching or leaving edit mode (a new `editing` object) reloads the fields.
  if (editing !== loaded) {
    loadFields(editing);
  }

  const odometerKm = parseKm(odometer);
  const litresCl = parseDecimal2(litres);
  const amountGrosze = parseDecimal2(amount);
  const canSubmit =
    !busy &&
    !requests.inFlight &&
    isIsoDate(date) &&
    odometerKm !== null &&
    litresCl !== null &&
    amountGrosze !== null;

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!canSubmit || odometerKm === null || litresCl === null || amountGrosze === null) return;
    onClearNotice();
    // Values above u32 would fail argument deserialization; they are out of range either way.
    if (odometerKm > U32_MAX) {
      setError(rejectionMessage({ kind: "odometerOutOfRange" }, startPoint));
      return;
    }
    if (litresCl > U32_MAX) {
      setError(rejectionMessage({ kind: "litresOutOfRange" }, startPoint));
      return;
    }
    const input = { fuel, date, odometerKm, litresCl, amountGrosze, fullTank };
    setBusy(true);
    setError("");
    try {
      if (editing) {
        const result = await requests.track(updateFillUp(editing.row.id, input));
        switch (result.kind) {
          case "saved":
            // Ending the edit reloads the fields into the add mode.
            onUpdated(editing.row.id, fuel, date, result.result);
            onSaved();
            break;
          case "notFound":
            loadFields(null);
            onGone(editing.row.id);
            setError(NOT_FOUND_MESSAGE);
            onSaved();
            break;
          case "rejected":
            setError(rejectionMessage(result.reason, startPoint));
            break;
          case "noStartPoint":
            onNoStartPoint();
            return;
        }
      } else {
        const result = await requests.track(addFillUp(input));
        switch (result.kind) {
          case "saved":
            onAdded(fuel, date, result.result);
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
      }
    } catch (err) {
      setError(errorMessage(err));
    }
    setBusy(false);
  }

  return (
    <section className="fill-up">
      <h2>{editing ? "Popraw tankowanie" : "Dodaj tankowanie"}</h2>
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
        {editing ? (
          <div className="fill-up-actions">
            <button type="submit" disabled={!canSubmit}>
              {busy ? "Zapisywanie…" : "Zapisz zmiany"}
            </button>
            <button type="button" disabled={busy} onClick={onCancelEditing}>
              Anuluj
            </button>
          </div>
        ) : (
          <button type="submit" disabled={!canSubmit}>
            {busy ? "Zapisywanie…" : "Zapisz tankowanie"}
          </button>
        )}
      </form>
      {notice && <FillUpResultPanel notice={notice} />}
    </section>
  );
}
