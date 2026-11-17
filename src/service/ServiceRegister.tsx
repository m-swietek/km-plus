import { useState } from "react";
import { formatDate, formatKm } from "../fuel/format";
import type { Recurrence, ServiceView } from "./api";

type Props = {
  /** In backend order; displayed as is. */
  services: ServiceView[];
  onEdit: (service: ServiceView) => void;
  /** Deletes after the in-row confirmation; resolves once the register has been refreshed. */
  onDelete: (id: number) => Promise<void>;
};

/** "przy 115 000 km", "do 01.03.2027" or "przy 180 000 km lub do 01.06.2028". */
function thresholdText(service: ServiceView): string {
  const parts: string[] = [];
  if (service.dueOdometerKm !== null) parts.push(`przy ${formatKm(service.dueOdometerKm)}`);
  if (service.dueDate !== null) parts.push(`do ${formatDate(service.dueDate)}`);
  return parts.join(" lub ");
}

/** "cyklicznie co 15 000 km", "cyklicznie co 12 mies." or both joined with "lub". */
function recurrenceText(recurrence: Recurrence): string {
  const parts: string[] = [];
  if (recurrence.everyKm !== null) parts.push(`co ${formatKm(recurrence.everyKm)}`);
  if (recurrence.everyMonths !== null) parts.push(`co ${recurrence.everyMonths} mies.`);
  return `cyklicznie ${parts.join(" lub ")}`;
}

/** The service register; "Usuń" asks for confirmation in the row, at most one row at a time. */
export function ServiceRegister({ services, onEdit, onDelete }: Props) {
  const [confirmingId, setConfirmingId] = useState<number | null>(null);
  const [deleting, setDeleting] = useState(false);

  async function confirmDelete(id: number) {
    setDeleting(true);
    await onDelete(id);
    setDeleting(false);
    setConfirmingId(null);
  }

  return (
    <section className="service-register">
      <h2>Rejestr serwisów</h2>
      {services.length === 0 ? (
        <p className="service-meta">Brak zaplanowanych serwisów</p>
      ) : (
        <ul className="service-register-list">
          {services.map((service) => (
            <li key={service.id} className="service-row">
              <p className="service-row-head">{service.name}</p>
              <p>{thresholdText(service)}</p>
              {service.recurrence && <p className="service-meta">{recurrenceText(service.recurrence)}</p>}
              {service.note && <p className="service-meta service-note">{service.note}</p>}
              {confirmingId === service.id ? (
                <div className="service-row-actions">
                  <span className="service-confirm">Usunąć serwis „{service.name}”?</span>
                  <button type="button" disabled={deleting} onClick={() => void confirmDelete(service.id)}>
                    Usuń
                  </button>
                  <button type="button" disabled={deleting} onClick={() => setConfirmingId(null)}>
                    Anuluj
                  </button>
                </div>
              ) : (
                <div className="service-row-actions">
                  <button type="button" disabled={deleting} onClick={() => onEdit(service)}>
                    Edytuj
                  </button>
                  <button type="button" disabled={deleting} onClick={() => setConfirmingId(service.id)}>
                    Usuń
                  </button>
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
