// Typed wrappers over the service register commands (src-tauri/src/service/commands.rs).
// Shapes are registered in docs/reference/contract-surfaces.md. Units: km (odometer reading and
// interval), months, dates as `YYYY-MM-DD`; every key is required, nullable values are `null`.
import { invoke } from "@tauri-apps/api/core";

/** Interval of a recurring service: set exactly for each threshold that is set. */
export type Recurrence = { everyKm: number | null; everyMonths: number | null };

/** One planned service in the register; at least one of `dueOdometerKm` / `dueDate` is set. */
export type ServiceView = {
  id: number;
  name: string;
  note: string | null;
  dueOdometerKm: number | null;
  dueDate: string | null;
  /** `null` for a one-off service. */
  recurrence: Recurrence | null;
};

/** `services` are in display order (dated first by date, then by km; ties by id). */
export type ServiceRegister = { services: ServiceView[] };

/** A service as entered; the backend trims `name` and `note` (an empty note is stored as `null`). */
export type ServiceInput = {
  name: string;
  note: string | null;
  dueOdometerKm: number | null;
  dueDate: string | null;
  recurrence: Recurrence | null;
};

export type ServiceRejection =
  | { kind: "nameEmpty" }
  | { kind: "nameTooLong" }
  | { kind: "noteTooLong" }
  | { kind: "noThreshold" }
  | { kind: "odometerOutOfRange" }
  | { kind: "invalidDate" }
  | { kind: "intervalMismatch" }
  | { kind: "intervalKmOutOfRange" }
  | { kind: "intervalMonthsOutOfRange" };

export type AddServiceResult =
  | { kind: "saved"; id: number }
  | { kind: "rejected"; reason: ServiceRejection };

export type UpdateServiceResult =
  | { kind: "saved" }
  | { kind: "notFound" }
  | { kind: "rejected"; reason: ServiceRejection };

export type DeleteServiceResult = { kind: "deleted" } | { kind: "notFound" };

export function serviceRegister(): Promise<ServiceRegister> {
  return invoke<ServiceRegister>("service_register");
}

export function addService(input: ServiceInput): Promise<AddServiceResult> {
  return invoke<AddServiceResult>("add_service", { input });
}

export function updateService(id: number, input: ServiceInput): Promise<UpdateServiceResult> {
  return invoke<UpdateServiceResult>("update_service", { id, input });
}

export function deleteService(id: number): Promise<DeleteServiceResult> {
  return invoke<DeleteServiceResult>("delete_service", { id });
}
