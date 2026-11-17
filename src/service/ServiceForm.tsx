import { useState, type FormEvent } from "react";
import { errorMessage } from "../vault/api";
import { U32_MAX } from "../fuel/api";
import { parseKm } from "../fuel/format";
import {
  addService,
  updateService,
  type ServiceInput,
  type ServiceRejection,
  type ServiceView,
} from "./api";

type Props = {
  /** The entry being edited, or `null` for a new one. The parent remounts the form (`key`) when this changes. */
  editing: ServiceView | null;
  /** A service was saved; `editedId` is the edited entry's id (`null` after adding). */
  onSaved: (editedId: number | null) => void;
  /** Editing was cancelled; the parent returns the form to add mode. */
  onCancel: () => void;
  /** The edited entry no longer exists in the backend. */
  onNotFound: (id: number) => void;
};

function rejectionMessage(reason: ServiceRejection): string {
  switch (reason.kind) {
    case "nameEmpty":
      return "Podaj nazwę serwisu.";
    case "nameTooLong":
      return "Nazwa może mieć najwyżej 100 znaków.";
    case "noteTooLong":
      return "Notatka może mieć najwyżej 500 znaków.";
    case "noThreshold":
      return "Podaj próg przebiegu, termin albo oba.";
    case "odometerOutOfRange":
      return "Próg przebiegu musi mieścić się w zakresie 1–9 999 999 km.";
    case "invalidDate":
      return "Nieprawidłowy termin.";
    case "intervalMismatch":
      return "Dla wpisu cyklicznego podaj interwał dla każdego zadanego progu.";
    case "intervalKmOutOfRange":
      return "Interwał przebiegu musi mieścić się w zakresie 1–9 999 999 km.";
    case "intervalMonthsOutOfRange":
      return "Interwał terminu musi mieścić się w zakresie 1–120 miesięcy.";
  }
}

/** Whole months: digits only (surrounding whitespace ignored). */
function parseMonths(text: string): number | null {
  const digits = text.trim();
  if (!/^\d+$/.test(digits)) return null;
  const value = Number(digits);
  return Number.isSafeInteger(value) ? value : null;
}

/** Add/edit form for a planned service. Thresholds are taken as entered; validation is the backend's. */
export function ServiceForm({ editing, onSaved, onCancel, onNotFound }: Props) {
  const [name, setName] = useState(editing?.name ?? "");
  const [note, setNote] = useState(editing?.note ?? "");
  const [odometer, setOdometer] = useState(editing?.dueOdometerKm?.toString() ?? "");
  const [date, setDate] = useState(editing?.dueDate ?? "");
  const [recurring, setRecurring] = useState(editing?.recurrence != null);
  const [everyKm, setEveryKm] = useState(editing?.recurrence?.everyKm?.toString() ?? "");
  const [everyMonths, setEveryMonths] = useState(editing?.recurrence?.everyMonths?.toString() ?? "");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  // An interval field is shown (and sent) only for a threshold that is filled in.
  const hasKmThreshold = odometer.trim().length > 0;
  const hasDateThreshold = date.length > 0;
  const showEveryKm = recurring && hasKmThreshold;
  const showEveryMonths = recurring && hasDateThreshold;

  const odometerKm = parseKm(odometer);
  const everyKmValue = parseKm(everyKm);
  const everyMonthsValue = parseMonths(everyMonths);
  // A filled-in field must parse; an empty one is sent as `null` and judged by the backend.
  const odometerBad = hasKmThreshold && odometerKm === null;
  const everyKmBad = showEveryKm && everyKm.trim().length > 0 && everyKmValue === null;
  const everyMonthsBad = showEveryMonths && everyMonths.trim().length > 0 && everyMonthsValue === null;
  const canSubmit = !busy && !odometerBad && !everyKmBad && !everyMonthsBad;

  function buildInput(): ServiceInput {
    // Every key is sent: the backend treats a missing key as an error, not as `null`.
    return {
      name,
      note: note.trim().length > 0 ? note : null,
      dueOdometerKm: hasKmThreshold ? odometerKm : null,
      dueDate: hasDateThreshold ? date : null,
      recurrence: recurring
        ? {
            everyKm: showEveryKm ? everyKmValue : null,
            everyMonths: showEveryMonths ? everyMonthsValue : null,
          }
        : null,
    };
  }

  function reset() {
    setName("");
    setNote("");
    setOdometer("");
    setDate("");
    setRecurring(false);
    setEveryKm("");
    setEveryMonths("");
  }

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!canSubmit) return;
    const input = buildInput();
    // Values above u32 would fail argument deserialization; they are out of range either way.
    if (input.dueOdometerKm !== null && input.dueOdometerKm > U32_MAX) {
      setError(rejectionMessage({ kind: "odometerOutOfRange" }));
      return;
    }
    if (input.recurrence?.everyKm != null && input.recurrence.everyKm > U32_MAX) {
      setError(rejectionMessage({ kind: "intervalKmOutOfRange" }));
      return;
    }
    if (input.recurrence?.everyMonths != null && input.recurrence.everyMonths > U32_MAX) {
      setError(rejectionMessage({ kind: "intervalMonthsOutOfRange" }));
      return;
    }
    setBusy(true);
    setError("");
    try {
      if (editing) {
        const result = await updateService(editing.id, input);
        switch (result.kind) {
          case "saved":
            reset();
            onSaved(editing.id);
            break;
          case "notFound":
            onNotFound(editing.id);
            break;
          case "rejected":
            setError(rejectionMessage(result.reason));
            break;
        }
      } else {
        const result = await addService(input);
        switch (result.kind) {
          case "saved":
            reset();
            onSaved(null);
            break;
          case "rejected":
            setError(rejectionMessage(result.reason));
            break;
        }
      }
    } catch (err) {
      setError(errorMessage(err));
    }
    setBusy(false);
  }

  return (
    <section className="service-form">
      <h2>{editing ? "Edytuj serwis" : "Dodaj serwis"}</h2>
      <form className="gate-form" onSubmit={handleSubmit}>
        <label htmlFor="service-name">Nazwa</label>
        <input
          id="service-name"
          autoComplete="off"
          value={name}
          readOnly={busy}
          onChange={(e) => setName(e.currentTarget.value)}
        />
        <label htmlFor="service-note">Notatka (opcjonalna)</label>
        <textarea
          id="service-note"
          rows={2}
          value={note}
          readOnly={busy}
          onChange={(e) => setNote(e.currentTarget.value)}
        />
        <label htmlFor="service-odometer">Próg przebiegu (km, opcjonalny)</label>
        <input
          id="service-odometer"
          inputMode="numeric"
          autoComplete="off"
          value={odometer}
          readOnly={busy}
          onChange={(e) => setOdometer(e.currentTarget.value)}
        />
        {odometerBad && <p className="gate-hint">Podaj przebieg w pełnych kilometrach (same cyfry).</p>}
        <label htmlFor="service-date">Termin (opcjonalny)</label>
        <input
          id="service-date"
          type="date"
          value={date}
          readOnly={busy}
          onChange={(e) => setDate(e.currentTarget.value)}
        />
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={recurring}
            disabled={busy}
            onChange={(e) => setRecurring(e.currentTarget.checked)}
          />
          Cykliczny
        </label>
        {showEveryKm && (
          <>
            <label htmlFor="service-every-km">Co ile km</label>
            <input
              id="service-every-km"
              inputMode="numeric"
              autoComplete="off"
              value={everyKm}
              readOnly={busy}
              onChange={(e) => setEveryKm(e.currentTarget.value)}
            />
            {everyKmBad && <p className="gate-hint">Podaj interwał w pełnych kilometrach (same cyfry).</p>}
          </>
        )}
        {showEveryMonths && (
          <>
            <label htmlFor="service-every-months">Co ile miesięcy</label>
            <input
              id="service-every-months"
              inputMode="numeric"
              autoComplete="off"
              value={everyMonths}
              readOnly={busy}
              onChange={(e) => setEveryMonths(e.currentTarget.value)}
            />
            {everyMonthsBad && <p className="gate-hint">Podaj liczbę pełnych miesięcy (same cyfry).</p>}
          </>
        )}
        {error && (
          <p className="gate-error" role="alert">
            {error}
          </p>
        )}
        <div className="service-form-actions">
          <button type="submit" disabled={!canSubmit}>
            {busy ? "Zapisywanie…" : editing ? "Zapisz zmiany" : "Zapisz serwis"}
          </button>
          {editing && (
            <button type="button" disabled={busy} onClick={onCancel}>
              Anuluj
            </button>
          )}
        </div>
      </form>
    </section>
  );
}
