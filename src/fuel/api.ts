// Typed wrappers over the fuel commands (src-tauri/src/fuel/commands.rs).
// Shapes are registered in docs/reference/contract-surfaces.md. Units: km, centilitres, grosze,
// dates as `YYYY-MM-DD`; `litresPer100Km` and `pricePerLitreGrosze` are unrounded floats.
import { invoke } from "@tauri-apps/api/core";

export type Fuel = "lpg" | "petrol";

export type StartPointView = { date: string; odometerKm: number };

export type LatestConsumption = {
  date: string;
  odometerKm: number;
  litresPer100Km: number;
  distanceKm: number;
  periodCostGrosze: number;
};

/** Average over every closed period of one fuel. */
export type AverageConsumption = {
  litresPer100Km: number;
  distanceKm: number;
  litresCl: number;
  periods: number;
};

export type FuelSummaryView =
  | { kind: "noEntries" }
  | { kind: "noClosedPeriod" }
  | { kind: "consumption"; latest: LatestConsumption; average: AverageConsumption };

/** One fill-up in the per-fuel history (no `fuel` field — the list is per fuel). */
export type HistoryRow = {
  id: number;
  date: string;
  odometerKm: number;
  litresCl: number;
  amountGrosze: number;
  fullTank: boolean;
  result: FillUpResult;
};

/** `history` is newest (highest odometer) first. */
export type FuelView = { summary: FuelSummaryView; history: HistoryRow[] };

/** `lpg`/`petrol` are `null` exactly when `startPoint` is `null`. */
export type FuelOverview = {
  startPoint: StartPointView | null;
  lpg: FuelView | null;
  petrol: FuelView | null;
};

export type StartPointRejection = "invalidDate" | "odometerOutOfRange";

export type SetStartPointResult =
  | { kind: "ok" }
  | { kind: "alreadySet" }
  | { kind: "invalid"; reason: { kind: StartPointRejection } };

export type FillUpInput = {
  fuel: Fuel;
  date: string;
  odometerKm: number;
  litresCl: number;
  amountGrosze: number;
  fullTank: boolean;
};

export type FillUpResult =
  | {
      kind: "consumption";
      litresPer100Km: number;
      distanceKm: number;
      periodLitresCl: number;
      periodCostGrosze: number;
      pricePerLitreGrosze: number;
    }
  | { kind: "topUp"; pricePerLitreGrosze: number };

export type FillUpRejection =
  | { kind: "invalidDate" }
  | { kind: "odometerOutOfRange" }
  | { kind: "litresOutOfRange" }
  | { kind: "amountOutOfRange" }
  | { kind: "dateBeforeStart" }
  | { kind: "odometerNotAfterStart" }
  | { kind: "sameFuelSameOdometer"; date: string }
  | { kind: "orderConflict"; date: string; odometerKm: number };

export type AddFillUpResult =
  | { kind: "saved"; result: FillUpResult }
  | { kind: "noStartPoint" }
  | { kind: "rejected"; reason: FillUpRejection };

/**
 * `odometerKm` and `litresCl` are `u32` in the backend; a larger number fails argument
 * deserialization with an opaque `Err`, so callers check this before invoking and report the
 * matching out-of-range rejection instead. Range validation proper stays in the backend.
 */
export const U32_MAX = 4_294_967_295;

export function fuelOverview(): Promise<FuelOverview> {
  return invoke<FuelOverview>("fuel_overview");
}

export function setStartPoint(date: string, odometerKm: number): Promise<SetStartPointResult> {
  return invoke<SetStartPointResult>("set_start_point", { date, odometerKm });
}

export function addFillUp(input: FillUpInput): Promise<AddFillUpResult> {
  return invoke<AddFillUpResult>("add_fill_up", { input });
}

export const FUEL_LABELS: Record<Fuel, string> = { lpg: "Gaz", petrol: "Benzyna" };
