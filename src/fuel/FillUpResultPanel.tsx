import { FUEL_LABELS } from "./api";
import { formatConsumption, formatDate, formatKm, formatMoney, formatPricePerLitre } from "./format";
import type { Notice } from "./useFillUpEditing";

type Props = {
  notice: Notice;
};

const TITLES: Record<Notice["kind"], string> = {
  added: "Zapisano",
  updated: "Poprawiono",
  deleted: "Usunięto",
};

/** Status of the last action: the saved or corrected entry with its result, or the deleted entry. */
export function FillUpResultPanel({ notice }: Props) {
  return (
    <div className="fill-up-result" role="status">
      <p className="fill-up-result-title">
        {TITLES[notice.kind]}: {FUEL_LABELS[notice.fuel]}, {formatDate(notice.date)}
        {notice.kind === "deleted" && <>, {formatKm(notice.odometerKm)}</>}
      </p>
      {notice.kind !== "deleted" &&
        (notice.result.kind === "consumption" ? (
          <p>
            Spalanie: <strong>{formatConsumption(notice.result.litresPer100Km)}</strong> na{" "}
            {formatKm(notice.result.distanceKm)} · Koszt okresu: {formatMoney(notice.result.periodCostGrosze)} · Cena:{" "}
            {formatPricePerLitre(notice.result.pricePerLitreGrosze)}
          </p>
        ) : (
          <p>
            Dolewka — brak wyniku spalania (okres zamknie tankowanie do pełna) · Cena:{" "}
            {formatPricePerLitre(notice.result.pricePerLitreGrosze)}
          </p>
        ))}
    </div>
  );
}
