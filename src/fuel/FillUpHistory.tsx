import { useState } from "react";
import { errorMessage } from "../vault/api";
import { FUEL_LABELS, deleteFillUp, type Fuel, type HistoryRow } from "./api";
import { formatConsumption, formatDate, formatKm, formatLitres, formatMoney, formatPricePerLitre } from "./format";

type Props = {
  fuel: Fuel;
  /** In backend order (newest first); displayed as is. */
  rows: HistoryRow[];
  /** Entry loaded into the form, highlighted in its list. */
  editingId: number | null;
  /** The one row on the whole screen asking for delete confirmation. */
  confirmingId: number | null;
  onEdit: (fuel: Fuel, row: HistoryRow) => void;
  onStartConfirming: (id: number) => void;
  onCancelConfirming: () => void;
  onDeleted: (fuel: Fuel, row: HistoryRow) => void;
  /** The entry no longer exists in the backend. */
  onGone: (id: number) => void;
  /** Data changed (or turned out stale); the shell refreshes the fuel cards and histories. */
  onChanged: () => void;
};

/**
 * Fill-up history of one fuel; a row that closed a period also shows its consumption. Each row can
 * be loaded into the form or deleted after a confirmation that replaces the row.
 */
export function FillUpHistory({
  fuel,
  rows,
  editingId,
  confirmingId,
  onEdit,
  onStartConfirming,
  onCancelConfirming,
  onDeleted,
  onGone,
  onChanged,
}: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  function startConfirming(id: number) {
    setError("");
    onStartConfirming(id);
  }

  async function confirmDelete(row: HistoryRow) {
    if (busy) return;
    setBusy(true);
    setError("");
    try {
      const result = await deleteFillUp(row.id);
      switch (result.kind) {
        case "deleted":
          onDeleted(fuel, row);
          break;
        case "notFound":
          // Already gone: the refresh removes the row; nothing else to report.
          onGone(row.id);
          break;
      }
      onChanged();
    } catch (err) {
      setError(errorMessage(err));
    }
    setBusy(false);
  }

  return (
    <section className="fuel-history">
      <h2>Historia — {FUEL_LABELS[fuel]}</h2>
      {rows.length === 0 ? (
        <p className="fuel-card-meta">Brak tankowań</p>
      ) : (
        <ol className="fuel-history-list">
          {rows.map((row) => (
            <li
              key={row.id}
              className={row.id === editingId ? "fuel-history-row fuel-history-row-editing" : "fuel-history-row"}
            >
              {row.id === confirmingId ? (
                <>
                  <p className="fuel-history-confirm">
                    Usunąć tankowanie z {formatDate(row.date)}, {formatKm(row.odometerKm)}?
                  </p>
                  {error && (
                    <p className="gate-error" role="alert">
                      {error}
                    </p>
                  )}
                  <div className="fuel-history-actions">
                    <button
                      type="button"
                      className="fuel-history-danger"
                      disabled={busy}
                      onClick={() => void confirmDelete(row)}
                    >
                      Usuń
                    </button>
                    <button type="button" disabled={busy} onClick={onCancelConfirming}>
                      Anuluj
                    </button>
                  </div>
                </>
              ) : (
                <>
                  <p className="fuel-history-head">
                    {formatDate(row.date)} · {formatKm(row.odometerKm)}
                  </p>
                  <p>
                    {formatLitres(row.litresCl)} · {formatMoney(row.amountGrosze)} ·{" "}
                    {formatPricePerLitre(row.result.pricePerLitreGrosze)}
                  </p>
                  <p className="fuel-card-meta">{row.fullTank ? "do pełna" : "dolewka"}</p>
                  {row.result.kind === "consumption" && (
                    <p>
                      <strong>{formatConsumption(row.result.litresPer100Km)}</strong> na{" "}
                      {formatKm(row.result.distanceKm)} · koszt okresu {formatMoney(row.result.periodCostGrosze)}
                    </p>
                  )}
                  <div className="fuel-history-actions">
                    <button type="button" disabled={busy} onClick={() => onEdit(fuel, row)}>
                      Edytuj
                    </button>
                    <button type="button" disabled={busy} onClick={() => startConfirming(row.id)}>
                      Usuń
                    </button>
                  </div>
                </>
              )}
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
