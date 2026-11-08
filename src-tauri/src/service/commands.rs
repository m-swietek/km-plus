//! Tauri command layer over the service register: reading the register and
//! adding, editing and deleting a planned service. Every command works only on
//! an unlocked vault session; writes go through `UnlockedVault::commit`, so a
//! failed save leaves the session data as it was.
//!
//! Work on the session (mutex lock, file write) runs on a blocking thread via
//! `tauri::async_runtime::spawn_blocking`, never on the async runtime.
//!
//! The frontend JSON shape is owned here (DTOs below, `kind`-tagged unions,
//! camelCase fields); the stored shape is owned by `super::PlannedService`.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use super::{
    register_order, validate_service, NewService, PlannedService, Recurrence, ServiceRejection,
};
use crate::vault::commands::{lock_session, VaultSession};
use crate::vault::{UnlockedVault, VaultData, VaultError};

const LOCKED_ERROR: &str = "dane są zablokowane; odblokuj aplikację";
const INTERNAL_ERROR: &str = "wewnętrzny błąd podczas operacji na danych";

/// The register in display order (see `super::register_order`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRegister {
    pub services: Vec<ServiceView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceView {
    pub id: u64,
    pub name: String,
    pub note: Option<String>,
    pub due_odometer_km: Option<u32>,
    pub due_date: Option<String>,
    pub recurrence: Option<RecurrenceView>,
}

/// Interval of a recurring service, both in `ServiceView` and `ServiceInput`
/// (JSON type `Recurrence`). Both keys are required, values may be `null`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurrenceView {
    #[serde(deserialize_with = "Option::deserialize")]
    pub every_km: Option<u32>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub every_months: Option<u32>,
}

/// A service as sent by the frontend (`add_service` / `update_service`
/// argument `input`). Every key is required, nullable values may be `null`,
/// so a frontend that forgets a field gets an `Err`, not a silently dropped
/// threshold.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInput {
    pub name: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub note: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub due_odometer_km: Option<u32>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub due_date: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub recurrence: Option<RecurrenceView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AddServiceResult {
    Saved { id: u64 },
    Rejected { reason: ServiceRejectionView },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UpdateServiceResult {
    Saved,
    NotFound,
    Rejected { reason: ServiceRejectionView },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum DeleteServiceResult {
    Deleted,
    NotFound,
}

/// JSON type `ServiceRejection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ServiceRejectionView {
    NameEmpty,
    NameTooLong,
    NoteTooLong,
    NoThreshold,
    OdometerOutOfRange,
    InvalidDate,
    IntervalMismatch,
    IntervalKmOutOfRange,
    IntervalMonthsOutOfRange,
}

impl From<ServiceRejection> for ServiceRejectionView {
    fn from(r: ServiceRejection) -> Self {
        match r {
            ServiceRejection::NameEmpty => Self::NameEmpty,
            ServiceRejection::NameTooLong => Self::NameTooLong,
            ServiceRejection::NoteTooLong => Self::NoteTooLong,
            ServiceRejection::NoThreshold => Self::NoThreshold,
            ServiceRejection::OdometerOutOfRange => Self::OdometerOutOfRange,
            ServiceRejection::InvalidDate => Self::InvalidDate,
            ServiceRejection::IntervalMismatch => Self::IntervalMismatch,
            ServiceRejection::IntervalKmOutOfRange => Self::IntervalKmOutOfRange,
            ServiceRejection::IntervalMonthsOutOfRange => Self::IntervalMonthsOutOfRange,
        }
    }
}

impl From<Recurrence> for RecurrenceView {
    fn from(r: Recurrence) -> Self {
        Self {
            every_km: r.every_km,
            every_months: r.every_months,
        }
    }
}

impl From<RecurrenceView> for Recurrence {
    fn from(r: RecurrenceView) -> Self {
        Self {
            every_km: r.every_km,
            every_months: r.every_months,
        }
    }
}

impl From<&PlannedService> for ServiceView {
    fn from(s: &PlannedService) -> Self {
        Self {
            id: s.id,
            name: s.name.clone(),
            note: s.note.clone(),
            due_odometer_km: s.due_odometer_km,
            due_date: s.due_date.clone(),
            recurrence: s.recurrence.clone().map(RecurrenceView::from),
        }
    }
}

impl From<ServiceInput> for NewService {
    fn from(i: ServiceInput) -> Self {
        NewService {
            name: i.name,
            note: i.note,
            due_odometer_km: i.due_odometer_km,
            due_date: i.due_date,
            recurrence: i.recurrence.map(Recurrence::from),
        }
    }
}

/// The service register in display order.
#[tauri::command]
pub async fn service_register(app: AppHandle) -> Result<ServiceRegister, String> {
    with_vault(app, |vault| Ok(register(&vault.data))).await
}

/// Validates and saves a new planned service, returning its id.
#[tauri::command]
pub async fn add_service(app: AppHandle, input: ServiceInput) -> Result<AddServiceResult, String> {
    with_vault(app, move |vault| apply_add_service(vault, input.into())).await
}

/// Replaces every field of the service with this `id`; the id stays.
#[tauri::command]
pub async fn update_service(
    app: AppHandle,
    id: u64,
    input: ServiceInput,
) -> Result<UpdateServiceResult, String> {
    with_vault(app, move |vault| apply_update_service(vault, id, input.into())).await
}

/// Removes the service with this `id`; its id is never reused.
#[tauri::command]
pub async fn delete_service(app: AppHandle, id: u64) -> Result<DeleteServiceResult, String> {
    with_vault(app, move |vault| apply_delete_service(vault, id)).await
}

/// Runs `f` on the unlocked vault on a blocking thread; `Err` when the
/// session is locked. The session is taken from the `AppHandle` inside the
/// closure because `State<'_, _>` is not `'static`.
async fn with_vault<T, F>(app: AppHandle, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut UnlockedVault) -> Result<T, VaultError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let session = app.state::<VaultSession>();
        let mut guard = lock_session(&session)?;
        let vault = guard.as_mut().ok_or_else(|| LOCKED_ERROR.to_string())?;
        f(vault).map_err(|e| e.to_string())
    })
    .await
    .map_err(|_| INTERNAL_ERROR.to_string())?
}

fn register(data: &VaultData) -> ServiceRegister {
    ServiceRegister {
        services: register_order(&data.services)
            .into_iter()
            .map(ServiceView::from)
            .collect(),
    }
}

fn apply_add_service(
    vault: &mut UnlockedVault,
    new: NewService,
) -> Result<AddServiceResult, VaultError> {
    let new = new.normalize();
    if let Err(reason) = validate_service(&new) {
        return Ok(AddServiceResult::Rejected {
            reason: reason.into(),
        });
    }
    let mut next = vault.data.clone();
    let id = next.next_service_id;
    next.next_service_id += 1;
    next.services.push(new.with_id(id));
    vault.commit(next)?;
    Ok(AddServiceResult::Saved { id })
}

/// An unknown id is reported before the input is validated: there is nothing
/// to correct in a form for an entry that no longer exists.
fn apply_update_service(
    vault: &mut UnlockedVault,
    id: u64,
    new: NewService,
) -> Result<UpdateServiceResult, VaultError> {
    let Some(index) = vault.data.services.iter().position(|s| s.id == id) else {
        return Ok(UpdateServiceResult::NotFound);
    };
    let new = new.normalize();
    if let Err(reason) = validate_service(&new) {
        return Ok(UpdateServiceResult::Rejected {
            reason: reason.into(),
        });
    }
    let mut next = vault.data.clone();
    next.services[index] = new.with_id(id);
    vault.commit(next)?;
    Ok(UpdateServiceResult::Saved)
}

fn apply_delete_service(
    vault: &mut UnlockedVault,
    id: u64,
) -> Result<DeleteServiceResult, VaultError> {
    let Some(index) = vault.data.services.iter().position(|s| s.id == id) else {
        return Ok(DeleteServiceResult::NotFound);
    };
    let mut next = vault.data.clone();
    next.services.remove(index);
    vault.commit(next)?;
    Ok(DeleteServiceResult::Deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::{KdfParams, UnlockOutcome, VAULT_FILE};
    use serde_json::{json, to_value};
    use std::path::Path;
    use tempfile::TempDir;

    const PASSWORD: &str = "correct horse";

    fn new_vault(dir: &Path) -> UnlockedVault {
        let cheap = KdfParams {
            m_cost: 8,
            t_cost: 1,
            p_cost: 1,
        };
        crate::vault::create(dir, PASSWORD, cheap).expect("create vault")
    }

    fn reopen(dir: &Path) -> UnlockedVault {
        match crate::vault::unlock(dir, PASSWORD).unwrap() {
            UnlockOutcome::Unlocked(v) => v,
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    fn vault_bytes(dir: &Path) -> Vec<u8> {
        std::fs::read(dir.join(VAULT_FILE)).expect("read vault file")
    }

    fn by_km(name: &str, km: u32) -> NewService {
        NewService {
            name: name.into(),
            note: None,
            due_odometer_km: Some(km),
            due_date: None,
            recurrence: None,
        }
    }

    fn by_date(name: &str, date: &str) -> NewService {
        NewService {
            name: name.into(),
            note: None,
            due_odometer_km: None,
            due_date: Some(date.into()),
            recurrence: None,
        }
    }

    fn names(register: &ServiceRegister) -> Vec<&str> {
        register.services.iter().map(|s| s.name.as_str()).collect()
    }

    #[track_caller]
    fn saved_id(result: AddServiceResult) -> u64 {
        match result {
            AddServiceResult::Saved { id } => id,
            other => panic!("expected saved, got {other:?}"),
        }
    }

    // --- JSON shape ---

    #[test]
    fn service_register_serializes_to_frontend_shape() {
        assert_eq!(
            to_value(ServiceRegister { services: vec![] }).unwrap(),
            json!({"services": []})
        );
        let register = ServiceRegister {
            services: vec![
                ServiceView {
                    id: 3,
                    name: "Rozrząd".into(),
                    note: Some("pasek + pompa".into()),
                    due_odometer_km: Some(180_000),
                    due_date: Some("2028-06-01".into()),
                    recurrence: Some(RecurrenceView {
                        every_km: Some(90_000),
                        every_months: Some(48),
                    }),
                },
                ServiceView {
                    id: 2,
                    name: "Wymiana oleju".into(),
                    note: None,
                    due_odometer_km: Some(115_000),
                    due_date: None,
                    recurrence: Some(RecurrenceView {
                        every_km: Some(15_000),
                        every_months: None,
                    }),
                },
                ServiceView {
                    id: 1,
                    name: "Przegląd".into(),
                    note: None,
                    due_odometer_km: None,
                    due_date: Some("2027-03-01".into()),
                    recurrence: None,
                },
            ],
        };
        assert_eq!(
            to_value(register).unwrap(),
            json!({"services": [
                {
                    "id": 3,
                    "name": "Rozrząd",
                    "note": "pasek + pompa",
                    "dueOdometerKm": 180_000,
                    "dueDate": "2028-06-01",
                    "recurrence": {"everyKm": 90_000, "everyMonths": 48}
                },
                {
                    "id": 2,
                    "name": "Wymiana oleju",
                    "note": null,
                    "dueOdometerKm": 115_000,
                    "dueDate": null,
                    "recurrence": {"everyKm": 15_000, "everyMonths": null}
                },
                {
                    "id": 1,
                    "name": "Przegląd",
                    "note": null,
                    "dueOdometerKm": null,
                    "dueDate": "2027-03-01",
                    "recurrence": null
                }
            ]})
        );
    }

    #[test]
    fn add_service_result_serializes_to_frontend_shape() {
        assert_eq!(
            to_value(AddServiceResult::Saved { id: 7 }).unwrap(),
            json!({"kind": "saved", "id": 7})
        );
        assert_eq!(
            to_value(AddServiceResult::Rejected {
                reason: ServiceRejectionView::NoThreshold
            })
            .unwrap(),
            json!({"kind": "rejected", "reason": {"kind": "noThreshold"}})
        );
    }

    #[test]
    fn update_service_result_serializes_to_frontend_shape() {
        assert_eq!(
            to_value(UpdateServiceResult::Saved).unwrap(),
            json!({"kind": "saved"})
        );
        assert_eq!(
            to_value(UpdateServiceResult::NotFound).unwrap(),
            json!({"kind": "notFound"})
        );
        assert_eq!(
            to_value(UpdateServiceResult::Rejected {
                reason: ServiceRejectionView::IntervalMismatch
            })
            .unwrap(),
            json!({"kind": "rejected", "reason": {"kind": "intervalMismatch"}})
        );
    }

    #[test]
    fn delete_service_result_serializes_to_frontend_shape() {
        assert_eq!(
            to_value(DeleteServiceResult::Deleted).unwrap(),
            json!({"kind": "deleted"})
        );
        assert_eq!(
            to_value(DeleteServiceResult::NotFound).unwrap(),
            json!({"kind": "notFound"})
        );
    }

    #[test]
    fn service_rejection_serializes_to_frontend_shape() {
        let all = [
            (ServiceRejection::NameEmpty, "nameEmpty"),
            (ServiceRejection::NameTooLong, "nameTooLong"),
            (ServiceRejection::NoteTooLong, "noteTooLong"),
            (ServiceRejection::NoThreshold, "noThreshold"),
            (ServiceRejection::OdometerOutOfRange, "odometerOutOfRange"),
            (ServiceRejection::InvalidDate, "invalidDate"),
            (ServiceRejection::IntervalMismatch, "intervalMismatch"),
            (ServiceRejection::IntervalKmOutOfRange, "intervalKmOutOfRange"),
            (
                ServiceRejection::IntervalMonthsOutOfRange,
                "intervalMonthsOutOfRange",
            ),
        ];
        for (rejection, kind) in all {
            assert_eq!(
                to_value(ServiceRejectionView::from(rejection)).unwrap(),
                json!({"kind": kind})
            );
        }
    }

    #[test]
    fn service_input_deserializes_from_frontend_shape() {
        let full: ServiceInput = serde_json::from_value(json!({
            "name": "Rozrząd",
            "note": "pasek + pompa",
            "dueOdometerKm": 180_000,
            "dueDate": "2028-06-01",
            "recurrence": {"everyKm": 90_000, "everyMonths": 48}
        }))
        .unwrap();
        assert_eq!(
            full,
            ServiceInput {
                name: "Rozrząd".into(),
                note: Some("pasek + pompa".into()),
                due_odometer_km: Some(180_000),
                due_date: Some("2028-06-01".into()),
                recurrence: Some(RecurrenceView {
                    every_km: Some(90_000),
                    every_months: Some(48),
                }),
            }
        );

        let nulls: ServiceInput = serde_json::from_value(json!({
            "name": "Przegląd",
            "note": null,
            "dueOdometerKm": null,
            "dueDate": "2027-03-01",
            "recurrence": null
        }))
        .unwrap();
        assert_eq!(
            nulls,
            ServiceInput {
                name: "Przegląd".into(),
                note: None,
                due_odometer_km: None,
                due_date: Some("2027-03-01".into()),
                recurrence: None,
            }
        );

        let null_interval: ServiceInput = serde_json::from_value(json!({
            "name": "Wymiana oleju",
            "note": null,
            "dueOdometerKm": 115_000,
            "dueDate": null,
            "recurrence": {"everyKm": 15_000, "everyMonths": null}
        }))
        .unwrap();
        assert_eq!(
            null_interval.recurrence,
            Some(RecurrenceView {
                every_km: Some(15_000),
                every_months: None,
            })
        );
    }

    #[test]
    fn service_input_requires_every_key() {
        let full = json!({
            "name": "Wymiana oleju",
            "note": null,
            "dueOdometerKm": 115_000,
            "dueDate": null,
            "recurrence": {"everyKm": 15_000, "everyMonths": null}
        });
        assert!(serde_json::from_value::<ServiceInput>(full.clone()).is_ok());
        for key in ["name", "note", "dueOdometerKm", "dueDate", "recurrence"] {
            let mut value = full.clone();
            value.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<ServiceInput>(value).is_err(),
                "missing {key}"
            );
        }
        for key in ["everyKm", "everyMonths"] {
            let mut value = full.clone();
            value["recurrence"].as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<ServiceInput>(value).is_err(),
                "missing recurrence.{key}"
            );
        }
    }

    #[test]
    fn service_input_converts_to_new_service() {
        let input = ServiceInput {
            name: "Rozrząd".into(),
            note: Some("pasek".into()),
            due_odometer_km: Some(180_000),
            due_date: Some("2028-06-01".into()),
            recurrence: Some(RecurrenceView {
                every_km: Some(90_000),
                every_months: Some(48),
            }),
        };
        assert_eq!(
            NewService::from(input),
            NewService {
                name: "Rozrząd".into(),
                note: Some("pasek".into()),
                due_odometer_km: Some(180_000),
                due_date: Some("2028-06-01".into()),
                recurrence: Some(Recurrence {
                    every_km: Some(90_000),
                    every_months: Some(48),
                }),
            }
        );
    }

    // --- Writes on a real vault ---

    #[test]
    fn added_services_get_increasing_ids_and_are_saved() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        // No start point needed: the gate is in the UI only.
        assert_eq!(vault.data.start_point, None);

        assert_eq!(
            apply_add_service(&mut vault, by_km("Wymiana oleju", 115_000)).unwrap(),
            AddServiceResult::Saved { id: 1 }
        );
        assert_eq!(
            apply_add_service(&mut vault, by_date("Przegląd", "2027-03-01")).unwrap(),
            AddServiceResult::Saved { id: 2 }
        );
        assert_eq!(vault.data.next_service_id, 3);

        let reopened = reopen(dir.path());
        assert_eq!(reopened.data.services, vault.data.services);
        assert_eq!(reopened.data.next_service_id, 3);
        let ids: Vec<u64> = reopened.data.services.iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn name_and_note_are_stored_normalized() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());

        let id = saved_id(
            apply_add_service(
                &mut vault,
                NewService {
                    name: "  Wymiana oleju \t".into(),
                    note: Some("\n 5W-30  ".into()),
                    ..by_km("", 115_000)
                },
            )
            .unwrap(),
        );
        let blank_note = saved_id(
            apply_add_service(
                &mut vault,
                NewService {
                    note: Some("   ".into()),
                    ..by_date(" Przegląd ", "2027-03-01")
                },
            )
            .unwrap(),
        );

        let stored = reopen(dir.path()).data.services;
        assert_eq!(stored[0].id, id);
        assert_eq!(stored[0].name, "Wymiana oleju");
        assert_eq!(stored[0].note.as_deref(), Some("5W-30"));
        assert_eq!(stored[1].id, blank_note);
        assert_eq!(stored[1].name, "Przegląd");
        assert_eq!(stored[1].note, None);

        // Editing normalizes the same way.
        assert_eq!(
            apply_update_service(
                &mut vault,
                id,
                NewService {
                    name: " Olej silnikowy ".into(),
                    note: Some(" \t ".into()),
                    ..by_km("", 116_000)
                },
            )
            .unwrap(),
            UpdateServiceResult::Saved
        );
        let edited = reopen(dir.path()).data.services;
        assert_eq!(edited[0].name, "Olej silnikowy");
        assert_eq!(edited[0].note, None);

        // A name of only whitespace is empty after trimming.
        assert_eq!(
            apply_add_service(&mut vault, by_km("   ", 115_000)).unwrap(),
            AddServiceResult::Rejected {
                reason: ServiceRejectionView::NameEmpty
            }
        );
    }

    #[test]
    fn update_keeps_id_and_replaces_every_field() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        let first = saved_id(
            apply_add_service(
                &mut vault,
                NewService {
                    note: Some("5W-30".into()),
                    recurrence: Some(Recurrence {
                        every_km: Some(15_000),
                        every_months: None,
                    }),
                    ..by_km("Wymiana oleju", 115_000)
                },
            )
            .unwrap(),
        );
        let second =
            saved_id(apply_add_service(&mut vault, by_date("Przegląd", "2027-03-01")).unwrap());

        let replacement = NewService {
            name: "Rozrząd".into(),
            note: None,
            due_odometer_km: Some(180_000),
            due_date: Some("2028-06-01".into()),
            recurrence: None,
        };
        assert_eq!(
            apply_update_service(&mut vault, first, replacement.clone()).unwrap(),
            UpdateServiceResult::Saved
        );

        let reopened = reopen(dir.path());
        assert_eq!(
            reopened.data.services,
            vec![
                replacement.with_id(first),
                by_date("Przegląd", "2027-03-01").with_id(second),
            ]
        );
        // Editing does not consume an id.
        assert_eq!(reopened.data.next_service_id, 3);
    }

    #[test]
    fn delete_removes_and_ids_are_not_reused() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        let first = saved_id(apply_add_service(&mut vault, by_km("A", 115_000)).unwrap());
        let second = saved_id(apply_add_service(&mut vault, by_km("B", 120_000)).unwrap());

        assert_eq!(
            apply_delete_service(&mut vault, second).unwrap(),
            DeleteServiceResult::Deleted
        );
        let reopened = reopen(dir.path());
        assert_eq!(
            reopened.data.services.iter().map(|s| s.id).collect::<Vec<_>>(),
            vec![first]
        );

        // The deleted (highest) id is not handed out again.
        assert_eq!(
            apply_add_service(&mut vault, by_km("C", 130_000)).unwrap(),
            AddServiceResult::Saved { id: 3 }
        );
        assert_eq!(
            apply_delete_service(&mut vault, first).unwrap(),
            DeleteServiceResult::Deleted
        );
        assert_eq!(
            apply_add_service(&mut vault, by_km("D", 140_000)).unwrap(),
            AddServiceResult::Saved { id: 4 }
        );
        let ids: Vec<u64> = reopen(dir.path()).data.services.iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![3, 4]);
    }

    #[test]
    fn unknown_id_is_not_found_and_writes_nothing() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        let id = saved_id(apply_add_service(&mut vault, by_km("A", 115_000)).unwrap());
        apply_delete_service(&mut vault, id).unwrap();
        let before_data = vault.data.clone();
        let before_bytes = vault_bytes(dir.path());

        for missing in [id, 99] {
            assert_eq!(
                apply_update_service(&mut vault, missing, by_km("B", 120_000)).unwrap(),
                UpdateServiceResult::NotFound
            );
            // Not found wins over an invalid input.
            assert_eq!(
                apply_update_service(&mut vault, missing, by_km("", 0)).unwrap(),
                UpdateServiceResult::NotFound
            );
            assert_eq!(
                apply_delete_service(&mut vault, missing).unwrap(),
                DeleteServiceResult::NotFound
            );
        }
        assert_eq!(vault.data, before_data);
        assert_eq!(vault_bytes(dir.path()), before_bytes);
    }

    #[test]
    fn rejection_writes_nothing() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        let id = saved_id(apply_add_service(&mut vault, by_km("A", 115_000)).unwrap());
        let before_data = vault.data.clone();
        let before_bytes = vault_bytes(dir.path());

        let no_threshold = NewService {
            due_odometer_km: None,
            ..by_km("B", 1)
        };
        assert_eq!(
            apply_add_service(&mut vault, no_threshold.clone()).unwrap(),
            AddServiceResult::Rejected {
                reason: ServiceRejectionView::NoThreshold
            }
        );
        assert_eq!(
            apply_update_service(&mut vault, id, no_threshold).unwrap(),
            UpdateServiceResult::Rejected {
                reason: ServiceRejectionView::NoThreshold
            }
        );
        assert_eq!(
            apply_update_service(
                &mut vault,
                id,
                NewService {
                    recurrence: Some(Recurrence {
                        every_km: None,
                        every_months: Some(12),
                    }),
                    ..by_km("A", 115_000)
                },
            )
            .unwrap(),
            UpdateServiceResult::Rejected {
                reason: ServiceRejectionView::IntervalMismatch
            }
        );
        assert_eq!(vault.data, before_data);
        assert_eq!(vault.data.next_service_id, 2);
        assert_eq!(vault_bytes(dir.path()), before_bytes);
    }

    #[test]
    fn register_returns_display_order() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        assert_eq!(register(&vault.data), ServiceRegister { services: vec![] });

        for new in [
            by_date("A", "2027-03-01"),
            by_km("B", 115_000),
            NewService {
                due_date: Some("2028-06-01".into()),
                ..by_km("C", 180_000)
            },
            by_date("D", "2026-10-01"),
            by_km("E", 115_000),
        ] {
            saved_id(apply_add_service(&mut vault, new).unwrap());
        }
        assert_eq!(names(&register(&vault.data)), ["D", "A", "C", "B", "E"]);
        assert_eq!(
            names(&register(&reopen(dir.path()).data)),
            ["D", "A", "C", "B", "E"]
        );
        assert_eq!(
            register(&vault.data).services[2],
            ServiceView {
                id: 3,
                name: "C".into(),
                note: None,
                due_odometer_km: Some(180_000),
                due_date: Some("2028-06-01".into()),
                recurrence: None,
            }
        );
    }
}
