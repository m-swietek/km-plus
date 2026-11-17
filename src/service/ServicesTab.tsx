import { useCallback, useEffect, useState } from "react";
import { errorMessage } from "../vault/api";
import { deleteService, serviceRegister, type ServiceView } from "./api";
import { ServiceForm } from "./ServiceForm";
import { ServiceRegister } from "./ServiceRegister";
import "./service.css";

type State =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "ready"; services: ServiceView[] };

const NOT_FOUND_MESSAGE = "Ten wpis już nie istnieje.";

/** "Serwisy" tab: owns the register, the id of the entry being edited, and refreshes after every write. */
export function ServicesTab() {
  const [state, setState] = useState<State>({ kind: "loading" });
  const [editingId, setEditingId] = useState<number | null>(null);
  const [notice, setNotice] = useState("");

  const fetchRegister = useCallback(async () => {
    try {
      setState({ kind: "ready", services: (await serviceRegister()).services });
    } catch (err) {
      setState({ kind: "error", message: errorMessage(err) });
    }
  }, []);

  /** Full reload with the loading screen. */
  const load = useCallback(async () => {
    setState({ kind: "loading" });
    await fetchRegister();
  }, [fetchRegister]);

  useEffect(() => {
    void load();
  }, [load]);

  /** Leaves edit mode only if the form is still on that entry (the user may have moved on meanwhile). */
  function stopEditing(id: number | null) {
    setEditingId((current) => (current === id ? null : current));
  }

  function handleSaved(editedId: number | null) {
    setNotice("");
    stopEditing(editedId);
    void fetchRegister();
  }

  function handleNotFound(id: number) {
    setNotice(NOT_FOUND_MESSAGE);
    stopEditing(id);
    void fetchRegister();
  }

  function handleEdit(service: ServiceView) {
    setNotice("");
    setEditingId(service.id);
  }

  async function handleDelete(id: number) {
    setNotice("");
    try {
      const result = await deleteService(id);
      if (result.kind === "notFound") setNotice(NOT_FOUND_MESSAGE);
      // Deleted or already gone: an edit of this entry has nothing left to save.
      stopEditing(id);
      await fetchRegister();
    } catch (err) {
      setNotice(errorMessage(err));
    }
  }

  switch (state.kind) {
    case "loading":
      return <p>Wczytywanie…</p>;

    case "error":
      return (
        <>
          <p className="gate-error" role="alert">
            {state.message}
          </p>
          <button type="button" onClick={load}>
            Spróbuj ponownie
          </button>
        </>
      );

    case "ready": {
      const editing = state.services.find((s) => s.id === editingId) ?? null;
      return (
        <>
          {/* Remounting on a different entry loads it into the form; refreshes keep the form mounted. */}
          <ServiceForm
            key={editing ? `edit-${editing.id}` : "add"}
            editing={editing}
            onSaved={handleSaved}
            onCancel={() => setEditingId(null)}
            onNotFound={handleNotFound}
          />
          {notice && (
            <p className="gate-error service-notice" role="alert">
              {notice}
            </p>
          )}
          <ServiceRegister
            services={state.services}
            onEdit={handleEdit}
            onDelete={handleDelete}
          />
        </>
      );
    }
  }
}
