import { FUEL_LABELS, type Fuel, type FillUpResult } from "./api";
import { formatConsumption, formatDate, formatKm, formatMoney, formatPricePerLitre } from "./format";

type Props = {
  fuel: Fuel;
  date: string;
  result: FillUpResult;
};

/** Result of the fill-up that was just saved. */
export function FillUpResultPanel({ fuel, date, result }: Props) {
  return (
    <div className="fill-up-result" role="status">
      <p className="fill-up-result-title">
        Zapisano: {FUEL_LABELS[fuel]}, {formatDate(date)}
      </p>
      {result.kind === "consumption" ? (
        <p>
          Spalanie: <strong>{formatConsumption(result.litresPer100Km)}</strong> na {formatKm(result.distanceKm)} ·
          Koszt okresu: {formatMoney(result.periodCostGrosze)} · Cena: {formatPricePerLitre(result.pricePerLitreGrosze)}
        </p>
      ) : (
        <p>
          Dolewka — brak wyniku spalania (okres zamknie tankowanie do pełna) · Cena:{" "}
          {formatPricePerLitre(result.pricePerLitreGrosze)}
        </p>
      )}
    </div>
  );
}
