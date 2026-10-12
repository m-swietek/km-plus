import { useCallback, useEffect, useState } from "react";
import { errorMessage } from "./vault/api";
import { FUEL_LABELS, fuelOverview, type Fuel, type FuelOverview, type LatestConsumption } from "./fuel/api";
import { formatConsumption, formatDate, formatKm } from "./fuel/format";
import { FillUpForm } from "./fuel/FillUpForm";
import { StartPointScreen } from "./fuel/StartPointScreen";

type State =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "ready"; overview: FuelOverview };

/** Application shell shown once the vault is unlocked: start point first, then fuel cards and the fill-up form. */
export function AppShell() {
  const [state, setState] = useState<State>({ kind: "loading" });

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
          <div className="fuel-cards">
            <FuelCard fuel="lpg" latest={overview.lpg} />
            <FuelCard fuel="petrol" latest={overview.petrol} />
          </div>
          {/* Refreshing in place keeps the form mounted, so its fuel, date and result survive. */}
          <FillUpForm startPoint={overview.startPoint} onSaved={fetchOverview} onNoStartPoint={load} />
        </section>
      );
    }
  }
}

function FuelCard({ fuel, latest }: { fuel: Fuel; latest: LatestConsumption | null }) {
  return (
    <div className="fuel-card">
      <h2>{FUEL_LABELS[fuel]}</h2>
      {latest ? (
        <>
          <p className="fuel-card-value">{formatConsumption(latest.litresPer100Km)}</p>
          <p className="fuel-card-meta">
            ostatnio {formatDate(latest.date)}, {formatKm(latest.odometerKm)}
          </p>
        </>
      ) : (
        <p className="fuel-card-meta">Brak wyniku — potrzebne tankowanie do pełna</p>
      )}
    </div>
  );
}
