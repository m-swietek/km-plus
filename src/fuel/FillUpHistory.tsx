import { FUEL_LABELS, type Fuel, type HistoryRow } from "./api";
import { formatConsumption, formatDate, formatKm, formatLitres, formatMoney, formatPricePerLitre } from "./format";

type Props = {
  fuel: Fuel;
  /** In backend order (newest first); displayed as is. */
  rows: HistoryRow[];
};

/** Fill-up history of one fuel; a row that closed a period also shows its consumption. */
export function FillUpHistory({ fuel, rows }: Props) {
  return (
    <section className="fuel-history">
      <h2>Historia — {FUEL_LABELS[fuel]}</h2>
      {rows.length === 0 ? (
        <p className="fuel-card-meta">Brak tankowań</p>
      ) : (
        <ol className="fuel-history-list">
          {rows.map((row) => (
            <li key={row.id} className="fuel-history-row">
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
                  <strong>{formatConsumption(row.result.litresPer100Km)}</strong> na {formatKm(row.result.distanceKm)}{" "}
                  · koszt okresu {formatMoney(row.result.periodCostGrosze)}
                </p>
              )}
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
