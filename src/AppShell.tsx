import { useCallback, useEffect, useState } from "react";
import { errorMessage } from "./vault/api";
import {
  FUEL_LABELS,
  fuelOverview,
  type Fuel,
  type FuelOverview,
  type FuelSummaryView,
  type FuelView,
} from "./fuel/api";
import { formatConsumption, formatDate, formatKm } from "./fuel/format";
import { FillUpForm } from "./fuel/FillUpForm";
import { FillUpHistory } from "./fuel/FillUpHistory";
import { StartPointScreen } from "./fuel/StartPointScreen";
import { useFillUpEditing } from "./fuel/useFillUpEditing";
import { ServicesTab } from "./service/ServicesTab";

type State =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "ready"; overview: FuelOverview };

type Tab = "fuel" | "services";

const TABS: { tab: Tab; label: string }[] = [
  { tab: "fuel", label: "Paliwo" },
  { tab: "services", label: "Serwisy" },
];

/**
 * Application shell shown once the vault is unlocked: start point first, then two tabs — "Paliwo"
 * (fuel cards, the fill-up form and per-fuel histories) and "Serwisy" (the service register).
 */
export function AppShell() {
  const [state, setState] = useState<State>({ kind: "loading" });
  const [tab, setTab] = useState<Tab>("fuel");
  const fillUpEditing = useFillUpEditing();

  const fetchOverview = useCallback(async () => {
    try {
      setState({ kind: "ready", overview: await fuelOverview() });
    } catch (err) {
      setState({ kind: "error", message: errorMessage(err) });
    }
  }, []);

  /** Full reload with the loading screen. */
  const load = useCallback(async () => {
    setState({ kind: "loading" });
    await fetchOverview();
  }, [fetchOverview]);

  useEffect(() => {
    void load();
  }, [load]);

  switch (state.kind) {
    case "loading":
      return (
        <section className="shell">
          <p>Wczytywanie…</p>
        </section>
      );

    case "error":
      return (
        <section className="shell">
          <h1>Wystąpił błąd</h1>
          <p className="gate-error" role="alert">
            {state.message}
          </p>
          <button type="button" onClick={load}>
            Spróbuj ponownie
          </button>
        </section>
      );

    case "ready": {
      const { overview } = state;
      if (!overview.startPoint) {
        return <StartPointScreen onDone={load} />;
      }
      return (
        <section className="shell">
          <h1>kmPlus</h1>
          <div className="tab-bar" role="tablist" aria-label="Sekcje">
            {TABS.map(({ tab: t, label }) => (
              <button
                key={t}
                type="button"
                role="tab"
                id={`tab-${t}`}
                aria-selected={tab === t}
                aria-controls={`tab-panel-${t}`}
                onClick={() => setTab(t)}
              >
                {label}
              </button>
            ))}
          </div>
          {/* Both tabs stay mounted and the inactive one is only hidden, so switching keeps every form's state. */}
          <div role="tabpanel" id="tab-panel-fuel" aria-labelledby="tab-fuel" hidden={tab !== "fuel"}>
            <div className="fuel-cards">
              <FuelCard fuel="lpg" view={overview.lpg} />
              <FuelCard fuel="petrol" view={overview.petrol} />
            </div>
            {/* Refreshing in place keeps the form mounted, so its fuel and date survive; the last-action notice lives in useFillUpEditing. */}
            <FillUpForm
              startPoint={overview.startPoint}
              onSaved={fetchOverview}
              onNoStartPoint={load}
              editing={fillUpEditing.editing}
              notice={fillUpEditing.notice}
              onCancelEditing={fillUpEditing.cancelEditing}
              onClearNotice={fillUpEditing.clearNotice}
              onAdded={fillUpEditing.reportAdded}
              onUpdated={fillUpEditing.reportUpdated}
              onGone={fillUpEditing.reportGone}
              requests={fillUpEditing.requests}
            />
            {/* Same overview state as the cards, so `fetchOverview` after a save refreshes both. */}
            <div className="fuel-histories">
              <FillUpHistory
                fuel="lpg"
                rows={overview.lpg?.history ?? []}
                editingId={fillUpEditing.editing?.row.id ?? null}
                confirmingId={fillUpEditing.confirmingId}
                onEdit={fillUpEditing.startEditing}
                onStartConfirming={fillUpEditing.startConfirming}
                onCancelConfirming={fillUpEditing.cancelConfirming}
                onDeleted={fillUpEditing.reportDeleted}
                onGone={fillUpEditing.reportGone}
                onChanged={fetchOverview}
                requests={fillUpEditing.requests}
              />
              <FillUpHistory
                fuel="petrol"
                rows={overview.petrol?.history ?? []}
                editingId={fillUpEditing.editing?.row.id ?? null}
                confirmingId={fillUpEditing.confirmingId}
                onEdit={fillUpEditing.startEditing}
                onStartConfirming={fillUpEditing.startConfirming}
                onCancelConfirming={fillUpEditing.cancelConfirming}
                onDeleted={fillUpEditing.reportDeleted}
                onGone={fillUpEditing.reportGone}
                onChanged={fetchOverview}
                requests={fillUpEditing.requests}
              />
            </div>
          </div>
          <div role="tabpanel" id="tab-panel-services" aria-labelledby="tab-services" hidden={tab !== "services"}>
            <ServicesTab />
          </div>
        </section>
      );
    }
  }
}

/** `view` is non-null whenever a start point exists; `null` is shown like an empty fuel. */
function FuelCard({ fuel, view }: { fuel: Fuel; view: FuelView | null }) {
  return (
    <div className="fuel-card">
      <h2>{FUEL_LABELS[fuel]}</h2>
      <FuelSummary summary={view?.summary ?? { kind: "noEntries" }} />
    </div>
  );
}

function FuelSummary({ summary }: { summary: FuelSummaryView }) {
  switch (summary.kind) {
    case "noEntries":
      return <p className="fuel-card-meta">Brak wpisów</p>;
    case "noClosedPeriod":
      return <p className="fuel-card-meta">Brak wyniku — potrzebne tankowanie do pełna</p>;
    case "consumption": {
      const { latest, average } = summary;
      return (
        <>
          <p className="fuel-card-value">{formatConsumption(latest.litresPer100Km)}</p>
          <p className="fuel-card-meta">
            ostatnio {formatDate(latest.date)}, {formatKm(latest.odometerKm)}
          </p>
          <p className="fuel-card-meta">
            średnio od startu: {formatConsumption(average.litresPer100Km)} na {formatKm(average.distanceKm)}
          </p>
        </>
      );
    }
  }
}
