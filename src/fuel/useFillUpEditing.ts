import { useState } from "react";
import type { Fuel, FillUpResult, HistoryRow } from "./api";

/** The entry loaded into the form; `fuel` is the list the row came from. */
export type Editing = { fuel: Fuel; row: HistoryRow };

/** The last action, shown in the status panel under the form. */
export type Notice =
  | { kind: "added" | "updated"; fuel: Fuel; date: string; result: FillUpResult }
  | { kind: "deleted"; fuel: Fuel; date: string; odometerKm: number };

/** Message shown when a corrected or deleted entry no longer exists (same text as the service register). */
export const NOT_FOUND_MESSAGE = "Ten wpis już nie istnieje.";

/**
 * State shared by the fill-up form and both histories: the entry being corrected, the one row
 * (on the whole screen) asking for delete confirmation, and the notice of the last action.
 */
export function useFillUpEditing() {
  const [editing, setEditing] = useState<Editing | null>(null);
  const [confirmingId, setConfirmingId] = useState<number | null>(null);
  const [notice, setNotice] = useState<Notice | null>(null);

  /** Loads a row into the form, replacing any unsaved draft; no stale notice or confirmation stays. */
  function startEditing(fuel: Fuel, row: HistoryRow) {
    setEditing({ fuel, row });
    setNotice(null);
    setConfirmingId(null);
  }

  function cancelEditing() {
    setEditing(null);
  }

  function startConfirming(id: number) {
    setConfirmingId(id);
  }

  function cancelConfirming() {
    setConfirmingId(null);
  }

  /** A submit is starting; the previous notice no longer describes the screen. */
  function clearNotice() {
    setNotice(null);
  }

  function reportAdded(fuel: Fuel, date: string, result: FillUpResult) {
    setNotice({ kind: "added", fuel, date, result });
  }

  function reportUpdated(fuel: Fuel, date: string, result: FillUpResult) {
    setNotice({ kind: "updated", fuel, date, result });
    setEditing(null);
  }

  /** Deleting the entry being corrected also ends the correction. */
  function reportDeleted(fuel: Fuel, row: HistoryRow) {
    setNotice({ kind: "deleted", fuel, date: row.date, odometerKm: row.odometerKm });
    setConfirmingId(null);
    setEditing((current) => (current?.row.id === row.id ? null : current));
  }

  /** The backend no longer has entry `id` (`notFound`): drop any edit or confirmation of it. */
  function reportGone(id: number) {
    setConfirmingId((current) => (current === id ? null : current));
    setEditing((current) => (current?.row.id === id ? null : current));
  }

  return {
    editing,
    confirmingId,
    notice,
    startEditing,
    cancelEditing,
    startConfirming,
    cancelConfirming,
    clearNotice,
    reportAdded,
    reportUpdated,
    reportDeleted,
    reportGone,
  };
}
